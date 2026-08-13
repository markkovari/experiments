//! The loop, end to end: a goal is attempted, judged by real commands, repaired
//! with what the commands actually said, and stopped for a reason.
//!
//! `agent.rs` proved a repair uses a failure it was HANDED. This proves the
//! failure gets there on its own: nothing in this test writes one. The check ids
//! are the only thing the scripted model matches on, and a check id only reaches
//! the model if the driver took it off a verdict that `comp-checks` produced by
//! running `grep`.
//!
//! Which is why the model is scripted and the checks are not. Feedback that was
//! planted proves nothing about the loop; a model that answers at random makes
//! "why did it stop after two attempts" a question with no reproducible answer.

use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use comp_reconciler::fleet::{bin_path, free_port, repo_root, Fleet};
use serde_json::{json, Value};

/// A native check runner that dies with the test.
struct Checks {
    child: Child,
    port: u16,
    _dir: tempfile::TempDir,
}

impl Drop for Checks {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Checks {
    fn start() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let port = free_port();
        let child = Command::new(bin_path("comp-checks"))
            .args(["--addr", &format!("127.0.0.1:{port}")])
            .arg("--work-dir")
            .arg(dir.path())
            .args(["--allow", "test", "--allow", "grep", "--timeout", "30"])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("comp-checks — run `cargo build --release` in reconciler/");
        let me = Self { child, port, _dir: dir };
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return me;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("comp-checks never listened");
    }
}

fn artifacts() -> Vec<String> {
    let dir = repo_root().join("components/target/wasm32-wasip2/release");
    let mut out = Vec::new();
    for (id, file) in [
        ("probe", "driver_probe.wasm"),
        ("driver", "agent_driver.wasm"),
        ("agent", "agent_writer.wasm"),
        ("llm", "mock_provider.wasm"),
        ("gate", "checks_runner.wasm"),
    ] {
        let p = dir.join(file);
        assert!(p.exists(), "missing {} — run `just build`", p.display());
        out.push(format!("{id}={}", p.display()));
    }
    out
}

fn spec_for(port: u16) -> std::path::PathBuf {
    let src = repo_root().join("fixtures/driver.yaml");
    let yaml = std::fs::read_to_string(&src).unwrap().replace("CHECKS_PORT", &port.to_string());
    let out = std::env::temp_dir().join(format!("comp-driver-{port}.yaml"));
    std::fs::write(&out, yaml).unwrap();
    out
}

struct Probe {
    port: u16,
    http: reqwest::blocking::Client,
}

impl Probe {
    fn run(&self, plan: Value) -> Value {
        let r = match self
            .http
            .post(format!("http://127.0.0.1:{}/run", self.port))
            .header("host", "driver.acme.test")
            .body(plan.to_string())
            .send()
        {
            Ok(r) => r,
            Err(e) => return Value::String(format!("transport: {e}")),
        };
        let (s, t) = (r.status(), r.text().unwrap_or_default());
        serde_json::from_str(&t).unwrap_or_else(|_| Value::String(format!("HTTP {s}: {t}")))
    }
}

const BASE: &str = "2222222222222222222222222222222222222222";

fn base_tree() -> Value {
    json!([
        { "path": "README", "content": "a project\n" },
        { "path": "src/lib.rs", "content": "pub fn answer() -> u32 { 41 }\n" },
    ])
}

fn check(id: &str, command: &[&str]) -> Value {
    json!({ "id": id, "required": true, "weight": 1, "command": command })
}

/// A plan with everything but the parts each case varies.
fn plan(text: &str, checks: Value, seed: u64, max_attempts: u32) -> Value {
    json!({
        "text": text,
        "writable": ["src/lib.rs"],
        "context": [{ "path": "src/lib.rs", "content": "pub fn answer() -> u32 { 41 }" }],
        "checks": checks,
        "base_commit": BASE,
        "base_tree": base_tree(),
        "max_attempts": max_attempts,
        "seed": seed,
    })
}

/// The first real run, retried until it works — not a separate readiness probe
/// (`Fleet::until`). Nothing shorter would do: a run that reaches the model but
/// not the runner produces an error rather than a hang, so the only signal that
/// the WHOLE chain is up is a run that came back with a scored attempt.
fn wait_for_probe(fleet: &Fleet) -> Probe {
    let probe = Probe {
        port: fleet.ingress_port,
        http: reqwest::blocking::Client::builder().timeout(Duration::from_secs(120)).build().unwrap(),
    };
    fleet.until("a run that reaches both the model and the runner", Duration::from_secs(180), || {
        let r = probe.run(plan(
            "go in circles",
            json!([check("never-passes", &["test", "-f", "never.txt"])]),
            0,
            1,
        ));
        if r["attempts"][0]["score"].is_number() { Ok(()) } else { Err(r.to_string()) }
    });
    probe
}

fn body_of(v: &Value) -> String {
    v["files"][0]["content"].as_str().unwrap_or_default().to_string()
}

#[test]
fn the_loop_repairs_from_a_real_verdict_and_stops_for_a_reason() {
    let checks = Checks::start();
    std::env::set_var("COMP_FLEET_ALLOW_PRIVATE_EGRESS", "1");
    let spec = spec_for(checks.port);
    let fleet = Fleet::start_with_secrets("driver", &[spec.to_str().unwrap()], &artifacts(), &[]);
    let probe = wait_for_probe(&fleet);

    // --- THE LOOP ------------------------------------------------------------
    // Attempt one writes 41 and `grep -q 42` fails. Nothing in this test tells
    // the model that: the only thing the script matches on is the check's id, and
    // that id reaches the model only if the driver lifted it off a verdict the
    // runner produced by running the command. Attempt two writes 42.
    let run = probe.run(plan(
        "make the answer 42",
        json!([
            check("base-arrived", &["test", "-f", "README"]),
            check("fix-the-answer", &["grep", "-q", "42", "src/lib.rs"]),
        ]),
        1,
        4,
    ));
    assert_eq!(run["stopped"], json!("accepted"), "the loop did not land it: {run}");
    assert_eq!(run["score"], json!(1000), "{run}");
    assert_eq!(body_of(&run), "pub fn answer() -> u32 { 42 }", "{run}");

    let attempts = run["attempts"].as_array().cloned().unwrap_or_default();
    assert_eq!(
        attempts.len(),
        2,
        "it should have taken exactly two: one to fail and one to fix. A run that took \
         one was never repaired, and a run that took four kept going after it had won: {run}"
    );
    assert_eq!(attempts[0]["score"], json!(500), "the first attempt got the base right and not the fix");
    assert_ne!(
        attempts[0]["digest"], attempts[1]["digest"],
        "the two attempts produced the same candidate, so the failure never reached the model \
         and the second attempt was a re-roll: {run}"
    );
    assert!(
        run["failures"].as_array().map_or(true, |f| f.is_empty()),
        "an accepted run has nothing left failing: {run}"
    );

    // --- STOPPING WITHOUT WINNING: the ideas ran out -------------------------
    // The model answers the same thing however many times it is asked. The
    // budget says five; the run stops at two, because a third would pay full
    // price for an answer already on record.
    let stuck = probe.run(plan(
        "go in circles",
        json!([check("never-passes", &["test", "-f", "never.txt"])]),
        0,
        5,
    ));
    assert_eq!(stuck["stopped"], json!("plateau"), "{stuck}");
    assert_eq!(stuck["accepted"], json!(false));
    assert_eq!(
        stuck["attempts"].as_array().map(|a| a.len()),
        Some(2),
        "a plateau that spends its whole budget is not a stopping rule: {stuck}"
    );
    let failures = stuck["failures"].as_array().cloned().unwrap_or_default();
    assert_eq!(
        failures[0]["id"],
        json!("never-passes"),
        "a run that found nothing must still say what was wrong, or a human has nothing \
         to act on: {stuck}"
    );

    // --- STOPPING WITHOUT WINNING: the budget ran out ------------------------
    // Three different candidates, none acceptable, two of them tied on score.
    // The one that is KEPT is the first to reach that score, not the last —
    // preferring the newer would make the answer depend on how many times it
    // happened to tie, and a repair that is worse than what it repaired would
    // walk the search backwards.
    let spent = probe.run(plan(
        "wander",
        json!([
            check("mention-answer", &["grep", "-q", "answer", "src/lib.rs"]),
            check("be-42", &["grep", "-q", "42", "src/lib.rs"]),
        ]),
        10,
        3,
    ));
    assert_eq!(spent["stopped"], json!("exhausted"), "{spent}");
    assert_eq!(spent["attempts"].as_array().map(|a| a.len()), Some(3), "{spent}");
    assert_eq!(spent["score"], json!(500), "half the gate: {spent}");
    assert_eq!(
        body_of(&spent),
        "pub fn answer() -> u32 { 41 }",
        "the run kept the wrong candidate — the best is the FIRST to reach the top score, \
         not the newest thing that tied it: {spent}"
    );

    // --- a run with no gate is refused before anything is paid for -----------
    // The evaluator refuses this too. Refused here as well so the caller learns
    // before spending on inference rather than after.
    let ungated = probe.run(plan("make the answer 42", json!([]), 1, 4));
    assert_eq!(ungated["error"], json!("invalid"), "an empty gate must be refused: {ungated}");

    println!(
        "    41 -> 42 in two attempts from a real verdict; plateau at 2 of 5; exhausted at 3, best kept"
    );
}
