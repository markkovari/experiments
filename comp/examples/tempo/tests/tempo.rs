//! E2E for the tempo worktime logger (TEMPO.md) as ONE composed wasm HTTP
//! component (tempo-domain + auth-guard + records) on the native Rust host.
//! Proves: auth + RBAC (admin creates projects/categories, member can't), time
//! logging + a pomodoro timer, and role-scoped range aggregation (a member sees
//! only their own totals; a manager sees the whole org, grouped by user).

use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

use serde_json::{json, Value};

const ADDR: &str = "127.0.0.1:3040";

struct HostGuard(Child);
impl Drop for HostGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn base() -> String {
    format!("http://{ADDR}")
}

fn req(method: &str, path: &str, token: Option<&str>, body: Option<Value>) -> (u16, Value) {
    let url = format!("{}{}", base(), path);
    let mut r = ureq::request(method, &url);
    if let Some(t) = token {
        r = r.set("authorization", &format!("Bearer {t}"));
    }
    let result = match &body {
        Some(b) => r.set("content-type", "application/json").send_string(&b.to_string()),
        None => r.call(),
    };
    let resp = match result {
        Ok(resp) => resp,
        Err(ureq::Error::Status(_, resp)) => resp,
        Err(e) => panic!("{method} {path}: {e}"),
    };
    let status = resp.status();
    (status, serde_json::from_str(&resp.into_string().unwrap_or_default()).unwrap_or(Value::Null))
}

fn signup(email: &str, role: &str) -> String {
    let (s, _) = req("POST", "/api/register", None, Some(json!({ "email": email, "password": "pw12345678", "role": role })));
    assert!(s == 201 || s == 409, "register {email}: {s}");
    let (s, l) = req("POST", "/api/login", None, Some(json!({ "email": email, "password": "pw12345678" })));
    assert_eq!(s, 200, "login {email}: {l}");
    l["access_token"].as_str().unwrap().to_string()
}

fn start_host() -> HostGuard {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let bin = root.join("host/target/release/vet-host");
    let component = root.join("components/target/tempo_domain.composed.wasm");
    assert!(bin.exists(), "host not built: {bin:?} (run `just e2e-tempo`)");
    assert!(component.exists(), "composed wasm missing (just compose-tempo)");
    let child = Command::new(&bin)
        .args(["--component", component.to_str().unwrap(), "--addr", ADDR, "--kv", "memory"])
        .env("VET_TENANT", "tempo")
        .spawn()
        .expect("spawn vet-host");
    let guard = HostGuard(child);
    for _ in 0..200 {
        if let Ok(r) = ureq::get(&base()).call() {
            if r.status() == 200 {
                return guard;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("tempo host did not start");
}

fn total(v: &Value) -> u64 {
    v["total_minutes"].as_u64().unwrap()
}

#[test]
fn worktime_logging_rbac_and_reports() {
    let _host = start_host();

    let admin = signup("admin@acme.io", "admin");
    let ada = signup("ada@acme.io", "member");
    let boss = signup("boss@acme.io", "manager");

    // ===== admin creates projects + categories; a member can't ==============
    let (s, proj) = req("POST", "/api/projects", Some(&admin), Some(json!({ "key": "APOLLO", "name": "Apollo" })));
    assert_eq!(s, 201, "create project: {proj}");
    let pid = proj["id"].as_str().unwrap().to_string();
    let (s, eng) = req("POST", "/api/categories", Some(&admin), Some(json!({ "name": "engineering" })));
    assert_eq!(s, 201, "{eng}");
    let eng_id = eng["id"].as_str().unwrap().to_string();
    let (_, sales) = req("POST", "/api/categories", Some(&admin), Some(json!({ "name": "sales" })));
    let sales_id = sales["id"].as_str().unwrap().to_string();

    let (s, _) = req("POST", "/api/projects", Some(&ada), Some(json!({ "key": "X", "name": "X" })));
    assert_eq!(s, 403, "a member cannot create projects");

    // ===== a member logs time ===============================================
    let log = |cat: &str, minutes: u64, day: &str| {
        let (s, e) = req("POST", "/api/entries", Some(&ada),
            Some(json!({ "project": pid, "category": cat, "minutes": minutes, "day": day })));
        assert_eq!(s, 201, "log: {e}");
    };
    log(&eng_id, 120, "2026-07-20");
    log(&sales_id, 60, "2026-07-21");
    // an unknown project is rejected
    let (s, _) = req("POST", "/api/entries", Some(&ada),
        Some(json!({ "project": "nope", "category": eng_id, "minutes": 30, "day": "2026-07-20" })));
    assert_eq!(s, 422, "unknown project rejected");

    // ===== the member's own report ==========================================
    let (s, r) = req("GET", "/api/report?from=2026-07-01&to=2026-07-31", Some(&ada), None);
    assert_eq!(s, 200, "{r}");
    assert_eq!(total(&r), 180, "member total: {r}");
    let by_proj = r["by_project"].as_array().unwrap();
    assert_eq!(by_proj.len(), 1);
    assert_eq!(by_proj[0]["name"], "Apollo");
    assert_eq!(by_proj[0]["minutes"], 180);
    let cats: Vec<(&str, u64)> = r["by_category"].as_array().unwrap().iter()
        .map(|c| (c["key"].as_str().unwrap(), c["minutes"].as_u64().unwrap())).collect();
    assert!(cats.contains(&("engineering", 120)) && cats.contains(&("sales", 60)), "by category: {cats:?}");

    // a member can't escalate scope — 'all' is forced back to 'me'
    let (_, r) = req("GET", "/api/report?from=2026-07-01&to=2026-07-31&scope=all", Some(&ada), None);
    assert_eq!(r["scope"], "me", "member scope stays me");
    assert_eq!(r["can_see_all"], false);

    // range filtering excludes out-of-range days
    let (_, r) = req("GET", "/api/report?from=2026-07-21&to=2026-07-31", Some(&ada), None);
    assert_eq!(total(&r), 60, "only the 21st: {r}");

    // ===== a manager sees the whole org, grouped by user ====================
    let (s, r) = req("GET", "/api/report?from=2026-07-01&to=2026-07-31&scope=all", Some(&boss), None);
    assert_eq!(s, 200, "{r}");
    assert_eq!(r["scope"], "all");
    assert_eq!(r["can_see_all"], true);
    assert_eq!(total(&r), 180, "manager sees the member's time: {r}");
    let users: Vec<&str> = r["by_user"].as_array().unwrap().iter().map(|u| u["key"].as_str().unwrap()).collect();
    assert!(users.contains(&"ada@acme.io"), "by_user has the member: {users:?}");

    // ===== the pomodoro timer ===============================================
    let (s, _) = req("POST", "/api/timer/start", Some(&ada),
        Some(json!({ "project": pid, "category": eng_id, "day": "2026-07-22" })));
    assert_eq!(s, 200);
    let (_, t) = req("GET", "/api/timer", Some(&ada), None);
    assert!(t["timer"]["started"].as_u64().is_some(), "timer running: {t}");
    let (s, entry) = req("POST", "/api/timer/stop", Some(&ada), None);
    assert_eq!(s, 201, "stop -> entry: {entry}");
    assert!(entry["minutes"].as_u64().unwrap() >= 1, "at least a minute");

    let (_, r) = req("GET", "/api/report?from=2026-07-01&to=2026-07-31", Some(&ada), None);
    assert!(total(&r) >= 181, "timer added to the total: {r}");
}
