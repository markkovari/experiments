//! Projects and the goal queue, through the real control plane.
//!
//! The lifecycle is the whole feature, and a lifecycle is only real if the
//! ILLEGAL moves are refused. A state field that anything may set to anything is
//! a state field, not a state machine — so most of this test is about the
//! transitions that must not happen: a goal reaching `done` without running, a
//! dead-lettered goal being quietly resurrected, two people starting the same
//! goal and both believing they own it.
//!
//! What is deliberately NOT here: a goal that actually does anything. Starting
//! one records that it started; what a run *does* needs the agent and the gate,
//! which do not exist yet (ADR-0082). Testing the queue is honest; pretending
//! there is something behind it would not be.

use std::time::Duration;

use comp_reconciler::fleet::Fleet;
use serde_json::{json, Value};

struct Api {
    base: String,
    http: reqwest::blocking::Client,
    token: String,
}

impl Api {
    fn new(base: String) -> Self {
        let http =
            reqwest::blocking::Client::builder().timeout(Duration::from_secs(30)).build().unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(90);
        while std::time::Instant::now() < deadline {
            if http.get(&base).send().is_ok() {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        let mut me = Self { base, http, token: String::new() };
        let body = json!({ "email": "ada@projects.test", "password": "password123" });
        let _ = me.raw("/api/register", body.clone());
        let v = me.raw("/api/login", body);
        me.token = v["token"].as_str().unwrap_or_default().to_string();
        assert!(!me.token.is_empty(), "could not log in: {v}");
        me
    }

    fn raw(&self, path: &str, body: Value) -> Value {
        self.http
            .post(format!("{}{path}", self.base))
            .json(&body)
            .send()
            .ok()
            .and_then(|r| r.json().ok())
            .unwrap_or(Value::Null)
    }

    fn post(&self, path: &str, body: Value) -> (u16, Value) {
        match self
            .http
            .post(format!("{}{path}", self.base))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
        {
            Ok(r) => (r.status().as_u16(), r.json().unwrap_or(Value::Null)),
            Err(e) => (0, Value::String(format!("transport: {e}"))),
        }
    }

    fn get(&self, path: &str) -> (u16, Value) {
        match self.http.get(format!("{}{path}", self.base)).bearer_auth(&self.token).send() {
            Ok(r) => (r.status().as_u16(), r.json().unwrap_or(Value::Null)),
            Err(e) => (0, Value::String(format!("transport: {e}"))),
        }
    }

    fn delete(&self, path: &str) -> (u16, Value) {
        match self.http.delete(format!("{}{path}", self.base)).bearer_auth(&self.token).send() {
            Ok(r) => (r.status().as_u16(), r.json().unwrap_or(Value::Null)),
            Err(e) => (0, Value::String(format!("transport: {e}"))),
        }
    }

    fn goal(&self, project: &str, title: &str) -> String {
        let (code, v) = self.post(&format!("/api/projects/{project}/goals"), json!({ "title": title }));
        assert_eq!(code, 201, "queueing `{title}` failed: {v}");
        v["id"].as_str().unwrap().to_string()
    }

    fn state_of(&self, project: &str, id: &str) -> String {
        let (_, v) = self.get(&format!("/api/projects/{project}/goals"));
        v["goals"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .find(|g| g["id"].as_str() == Some(id))
            .map(|g| g["state"].as_str().unwrap_or_default().to_string())
            .unwrap_or_else(|| "(gone)".into())
    }
}

#[test]
fn a_queue_that_only_a_person_can_start_and_that_refuses_illegal_moves() {
    let fleet = Fleet::start_with_platform("projects", 1);
    let api = Api::new(fleet.platform_url());

    // --- a project owns one repo --------------------------------------------
    let (code, p) = api.post("/api/projects", json!({ "name": "widgets", "repo": "acme/widgets" }));
    assert_eq!(code, 201, "creating a project failed: {p}");
    assert_eq!(p["base"], json!("main"), "the base should default: {p}");

    // A name that would not survive being part of a store or branch name, and a
    // repo that is not `owner/name`, are refused HERE — where the message can say
    // so — rather than at the first forge call, which answers 404.
    for bad in [
        json!({ "name": "Widgets", "repo": "acme/widgets" }),
        json!({ "name": "-lead", "repo": "acme/widgets" }),
        json!({ "name": "ok", "repo": "widgets" }),
        json!({ "name": "ok", "repo": "a/b/c" }),
    ] {
        let (code, v) = api.post("/api/projects", bad.clone());
        assert_eq!(code, 422, "{bad} should have been refused: {v}");
    }
    let (code, v) = api.post("/api/projects", json!({ "name": "widgets", "repo": "acme/other" }));
    assert_eq!(code, 409, "a duplicate project name should conflict: {v}");

    // --- the queue ----------------------------------------------------------
    let cache = api.goal("widgets", "add a cache");
    let rename = api.goal("widgets", "rename the thing");
    let doomed = api.goal("widgets", "something impossible");

    let (_, v) = api.get("/api/projects/widgets/goals");
    assert_eq!(v["count"], json!(3), "three goals should be queued: {v}");
    assert!(
        v["goals"].as_array().unwrap().iter().all(|g| g["state"] == json!("queued")),
        "everything starts queued and STAYS there — nothing drains this: {v}"
    );

    // The queue does not move on its own. Waiting proves it: a loop that drained
    // would have taken something by now, and this design deliberately has none.
    std::thread::sleep(Duration::from_secs(6));
    assert_eq!(
        api.state_of("widgets", &cache),
        "queued",
        "something started a goal without being asked — a human starts every goal"
    );

    // --- the one transition a person makes ----------------------------------
    let (code, v) = api.post(&format!("/api/goals/{cache}/start"), json!({}));
    assert_eq!(code, 200, "starting failed: {v}");
    assert_eq!(v["from"], json!("queued"));
    assert_eq!(api.state_of("widgets", &cache), "running");

    // --- the illegal moves, which are the point -----------------------------
    // Straight to done without ever having been reviewed.
    let (code, v) = api.post(&format!("/api/goals/{cache}/done"), json!({}));
    assert_eq!(code, 409, "a running goal must not jump to done: {v}");

    // Started twice. With one run per project this is the case the whole design
    // exists to prevent, and the record's revision is what prevents it.
    let (code, v) = api.post(&format!("/api/goals/{cache}/start"), json!({}));
    assert_eq!(code, 409, "a goal already running must not start again: {v}");

    // The legal path through review.
    let (code, v) = api.post(&format!("/api/goals/{cache}/review"), json!({}));
    assert_eq!(code, 200, "running -> awaiting-human should be legal: {v}");
    let (code, v) = api.post(&format!("/api/goals/{cache}/done"), json!({}));
    assert_eq!(code, 200, "awaiting-human -> done should be legal: {v}");
    assert_eq!(api.state_of("widgets", &cache), "done");

    // --- the dead-letter queue is terminal ----------------------------------
    let (code, _) = api.post(&format!("/api/goals/{doomed}/start"), json!({}));
    assert_eq!(code, 200);
    let (code, v) = api.post(
        &format!("/api/goals/{doomed}/fail"),
        json!({ "reason": "the spec asked for something that cannot exist" }),
    );
    assert_eq!(code, 200, "failing should be allowed: {v}");

    let (_, v) = api.get("/api/projects/widgets/goals?state=failed");
    assert_eq!(v["count"], json!(1), "the dead-letter queue should hold it: {v}");
    let dead = &v["goals"][0];
    assert!(
        dead["reason"].as_str().unwrap_or_default().contains("cannot exist"),
        "a dead letter with no reason is one nobody can act on: {dead}"
    );

    // Nothing leaves `failed`. A retry is a NEW goal, so what was tried stays
    // visible — resurrecting this one would erase the history of the attempt.
    for to in ["start", "review", "done"] {
        let (code, v) = api.post(&format!("/api/goals/{doomed}/{to}"), json!({}));
        assert_eq!(code, 409, "a dead-lettered goal must not be resurrected via {to}: {v}");
    }

    // --- abandoning something never started ---------------------------------
    let (code, v) = api.delete(&format!("/api/goals/{rename}"));
    assert_eq!(code, 200, "abandoning a queued goal should work: {v}");
    assert_eq!(api.state_of("widgets", &rename), "abandoned");

    // --- what the listing tells a person ------------------------------------
    let (_, v) = api.get("/api/projects");
    let widgets = v["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == json!("widgets"))
        .cloned()
        .unwrap();
    assert_eq!(widgets["queued"], json!(0), "nothing left queued: {widgets}");
    assert_eq!(widgets["failed"], json!(1), "one dead letter: {widgets}");
    assert_eq!(widgets["repo"], json!("acme/widgets"));

    // A goal for a project that does not exist is a 404, not a goal filed under a
    // typo that nobody will ever look at.
    let (code, v) = api.post("/api/projects/nosuch/goals", json!({ "title": "x" }));
    assert_eq!(code, 404, "a goal needs a project that exists: {v}");

    println!("    a queue nothing drains, and six illegal transitions refused");
}
