//! A generation: one goal, N branches at once, one winner.
//!
//! ## Why this is native and not a component
//!
//! Because it is the part that has to happen CONCURRENTLY. A component runs one
//! call at a time; a generation whose branches ran in sequence would take N times
//! as long as its slowest branch and would not be a generation at all — it would
//! be a loop with extra vocabulary. The fan-out is threads and sockets, so it
//! lives where threads and sockets live, and every decision it makes is delegated
//! to a component that can be reasoned about: the driver decides when a branch
//! stops, the selector decides which branch won.
//!
//! ## Seeds are spaced, not consecutive
//!
//! Attempt `n` of a branch uses `seed + n`, so branches seeded one apart would
//! share prompts: branch 1's second attempt and branch 2's first would ask the
//! same question with the same seed. `STRIDE` keeps them apart, and it is far
//! larger than any sane `max-attempts` so the overlap cannot creep back.
//!
//! ## What a failed branch is
//!
//! An ordinary result. A branch that could not reach its provider still returns
//! an entry — unaccepted, with the error in `note` — because a generation of four
//! in which one died is a generation of three, not a failure. Dropping it
//! silently would make `distinct` and the total cost quietly wrong.

use std::time::{Duration, Instant};

use serde_json::{json, Value};

/// The gap between one branch's seed and the next.
///
/// Attempt `n` uses `seed + n`. Anything smaller than `max-attempts` makes two
/// branches ask an identical question, which is the one thing a generation is
/// for avoiding.
pub const STRIDE: u64 = 100;

/// What one branch came back with, in the shape `graph:select` wants.
#[derive(Clone, Debug)]
pub struct Entry {
    pub branch: String,
    pub accepted: bool,
    pub score: u64,
    pub digest: String,
    pub spent_tokens: u64,
    pub attempts: u64,
    pub files: Value,
    /// Why this branch produced nothing, when it produced nothing.
    pub note: String,
    /// How long this branch took on its own.
    ///
    /// The only way to tell a fan-out from a for-loop after the fact: run in
    /// parallel the wall clock is about the slowest branch, run in sequence it is
    /// about the SUM. Counting attempts cannot distinguish them, which a
    /// deliberately sequential version of `fan_out` proved by passing.
    pub elapsed_ms: u64,
    /// What the driver reported as its reason for stopping. Not sent to the
    /// selector — how a branch ended is not a property of the code it wrote —
    /// but the most useful thing in the log when a generation finds nothing.
    pub stopped: String,
}

impl Entry {
    pub fn as_json(&self) -> Value {
        json!({
            "branch": self.branch,
            "accepted": self.accepted,
            "score": self.score,
            "digest": self.digest,
            "spent_tokens": self.spent_tokens,
            "attempts": self.attempts,
            "files": self.files,
        })
    }
}

fn client(timeout: Duration) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder().timeout(timeout).build().expect("http client")
}

fn post(url: &str, host: &str, body: &Value, timeout: Duration) -> Result<Value, String> {
    let r = client(timeout)
        .post(url)
        .header("host", host)
        .body(body.to_string())
        .send()
        .map_err(|e| format!("{e}"))?;
    let (status, text) = (r.status(), r.text().unwrap_or_default());
    serde_json::from_str(&text).map_err(|_| format!("HTTP {status}: {text}"))
}

/// Run one branch and turn whatever came back into an entry.
fn one_branch(url: &str, host: &str, plan: &Value, name: &str, seed: u64, timeout: Duration) -> Entry {
    let mut plan = plan.clone();
    plan["seed"] = json!(seed);

    let started = Instant::now();
    let blank = |note: String, stopped: &str| Entry {
        branch: name.to_string(),
        accepted: false,
        score: 0,
        digest: String::new(),
        spent_tokens: 0,
        attempts: 0,
        files: json!([]),
        note,
        elapsed_ms: started.elapsed().as_millis() as u64,
        stopped: stopped.to_string(),
    };

    let answer = match post(url, host, &plan, timeout) {
        Ok(v) => v,
        // A branch that could not be reached is a branch that found nothing. The
        // generation carries on with the rest — which is the entire argument for
        // running more than one.
        Err(e) => return blank(e, "unreachable"),
    };
    if let Some(err) = answer["error"].as_str() {
        return blank(
            format!("{err}: {}", answer["detail"].as_str().unwrap_or_default()),
            err,
        );
    }

    // The digest of the run's BEST candidate, which is what the selector compares
    // across branches. The driver reports one per attempt; the one that matters
    // is the attempt whose score the run kept.
    let attempts = answer["attempts"].as_array().cloned().unwrap_or_default();
    let score = answer["score"].as_u64().unwrap_or(0);
    let digest = attempts
        .iter()
        .find(|a| a["score"].as_u64() == Some(score) && !a["digest"].as_str().unwrap_or("").is_empty())
        .and_then(|a| a["digest"].as_str())
        .unwrap_or_default()
        .to_string();

    Entry {
        branch: name.to_string(),
        accepted: answer["accepted"].as_bool().unwrap_or(false),
        score,
        digest,
        spent_tokens: answer["spent_tokens"].as_u64().unwrap_or(0),
        attempts: attempts.len() as u64,
        files: answer["files"].clone(),
        note: String::new(),
        elapsed_ms: started.elapsed().as_millis() as u64,
        stopped: answer["stopped"].as_str().unwrap_or_default().to_string(),
    }
}

/// Fan one plan out to `branches` branches and wait for all of them.
///
/// Every branch is waited for, including the slow ones. Taking the first N to
/// finish would systematically prefer the branches that gave up early, which is
/// the opposite of what a search wants.
pub fn fan_out(
    driver_url: &str,
    host: &str,
    plan: &Value,
    branches: u16,
    base_seed: u64,
    timeout: Duration,
) -> Vec<Entry> {
    let handles: Vec<_> = (0..branches)
        .map(|i| {
            let (url, host, plan) = (driver_url.to_string(), host.to_string(), plan.clone());
            let name = format!("branch-{i}");
            let seed = base_seed + (i as u64) * STRIDE;
            std::thread::spawn(move || one_branch(&url, &host, &plan, &name, seed, timeout))
        })
        .collect();

    handles
        .into_iter()
        .enumerate()
        .map(|(i, h)| {
            h.join().unwrap_or_else(|_| Entry {
                branch: format!("branch-{i}"),
                accepted: false,
                score: 0,
                digest: String::new(),
                spent_tokens: 0,
                attempts: 0,
                files: json!([]),
                note: "the branch panicked".into(),
                elapsed_ms: 0,
                stopped: "panic".into(),
            })
        })
        .collect()
}

/// Hand a generation's entries to the selector, which decides and proposes.
///
/// The entries go across whole. Filtering the unaccepted ones out here would put
/// the gate in two places, and the one that could be forgotten is this one.
pub fn land(
    select_url: &str,
    host: &str,
    entries: &[Entry],
    landing: Value,
    timeout: Duration,
) -> Result<Value, String> {
    post(
        select_url,
        host,
        &json!({
            "entries": entries.iter().map(Entry::as_json).collect::<Vec<_>>(),
            "landing": landing,
        }),
        timeout,
    )
}
