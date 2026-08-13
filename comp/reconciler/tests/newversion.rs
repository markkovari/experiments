//! Recompile, upload, and watch the fleet stop running the old code.
//!
//! Everything else here tests DEPLOYING an artifact. This tests REPLACING one,
//! which is the harder half and the one a graph loop needs: an agent that
//! produces a change has to be able to ship it, and "shipped" means the node is
//! running the new bytes rather than the record saying so.
//!
//! The component answers with a tag baked in at compile time, so what is actually
//! running is readable from outside. A version field the platform sets is a claim;
//! a string the running code emits is evidence.
//!
//! It really recompiles. `cargo build` runs inside the test with a different
//! `COMP_VERSION_TAG`, and the test refuses to continue if the two builds produce
//! the same bytes — because a cache hit would make everything below it pass while
//! proving nothing.

use std::process::Command;
use std::time::{Duration, Instant};

use comp_reconciler::fleet::{repo_root, Fleet};
use serde_json::{json, Value};

/// Build the probe with a tag, and hand back the bytes.
fn build(tag: &str) -> Vec<u8> {
    let components = repo_root().join("components");
    let out = Command::new("cargo")
        .current_dir(&components)
        .args(["build", "--release", "--target", "wasm32-wasip2", "-p", "version-probe"])
        .env("COMP_VERSION_TAG", tag)
        .output()
        .expect("running cargo");
    assert!(
        out.status.success(),
        "building {tag} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let path = components.join("target/wasm32-wasip2/release/version_probe.wasm");
    std::fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

struct Api {
    base: String,
    http: reqwest::blocking::Client,
    token: String,
}

impl Api {
    fn new(base: String) -> Self {
        let http =
            reqwest::blocking::Client::builder().timeout(Duration::from_secs(60)).build().unwrap();
        let deadline = Instant::now() + Duration::from_secs(120);
        while Instant::now() < deadline {
            if http.get(&base).send().is_ok() {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        let mut me = Self { base, http, token: String::new() };
        let body = json!({ "email": "ada@version.test", "password": "password123" });
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

    /// Upload an artifact under an id. The digest is the platform's business —
    /// the caller sends bytes.
    fn get(&self, path: &str) -> Value {
        self.http
            .get(format!("{}{path}", self.base))
            .bearer_auth(&self.token)
            .send()
            .ok()
            .and_then(|r| r.json().ok())
            .unwrap_or(Value::Null)
    }

    /// The digest the platform currently believes this component has. The whole
    /// question in a version test is whether this MOVED.
    fn digest_of(&self, id: &str) -> String {
        let v = self.get("/api/components");
        v["components"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .find(|c| c["id"].as_str() == Some(id))
            .map(|c| c["oci_ref"].as_str().unwrap_or("(none)").to_string())
            .unwrap_or_else(|| "(not in the catalogue)".into())
    }

    /// The digest in the CURRENT manifest — what the fleet is actually told to
    /// run. For a fused deployment this is the composed artifact rather than the
    /// uploaded component, which is why it is the number that matters.
    fn manifest_digest(&self, id: &str) -> String {
        self.get(&format!("/api/deployments/{id}/manifests"))["manifest"]["components"][0]
            ["digest"]
            .as_str()
            .unwrap_or("(none)")
            .to_string()
    }

    fn upload(&self, id: &str, wasm: Vec<u8>) -> u16 {
        self.http
            .post(format!("{}/api/components?id={id}", self.base))
            .bearer_auth(&self.token)
            .body(wasm)
            .send()
            .unwrap()
            .status()
            .as_u16()
    }
}

/// What the app is serving right now, via the ingress. `None` until it answers.
///
/// The host is `{app}.{org}.{suffix}` — the platform derives it, nobody chooses
/// it, and guessing it wrong reads exactly like the app never starting.
fn tag_served(fleet: &Fleet) -> Option<String> {
    let http = reqwest::blocking::Client::builder().timeout(Duration::from_secs(10)).build().unwrap();
    let r = http
        .get(format!("http://127.0.0.1:{}/", fleet.ingress_port))
        .header("host", "ver.ada.test")
        .send()
        .ok()?;
    let v: Value = serde_json::from_str(&r.text().ok()?).ok()?;
    v["tag"].as_str().map(str::to_string)
}

fn wait_for_tag(fleet: &Fleet, want: &str, within: Duration) -> bool {
    let deadline = Instant::now() + within;
    while Instant::now() < deadline {
        if tag_served(fleet).as_deref() == Some(want) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    false
}

#[test]
fn a_recompiled_artifact_replaces_the_running_one() {
    // --- the premise, checked before anything depends on it ------------------
    let alpha = build("alpha");
    let beta = build("beta");
    assert_ne!(
        alpha, beta,
        "two builds with different tags produced identical bytes — the rebuild was a \
         cache hit, so everything below this would pass while shipping nothing. That is \
         what `build.rs`'s rerun-if-env-changed is for."
    );
    println!("    alpha {} bytes, beta {} bytes, and they differ", alpha.len(), beta.len());

    let fleet = Fleet::start_with_platform("newversion", 1);
    let api = Api::new(fleet.platform_url());

    // --- ship v1 -------------------------------------------------------------
    assert!(matches!(api.upload("ver", alpha.clone()), 200 | 201), "uploading alpha failed");
    let (code, dep) = api.post(
        "/api/deployments",
        json!({ "name": "ver", "nodes": [{"id": "ver"}], "edges": [] }),
    );
    assert_eq!(code, 201, "deploy failed: {dep}");
    let id = dep["id"].as_str().unwrap().to_string();

    let deadline = Instant::now() + Duration::from_secs(180);
    let mut saved = false;
    let mut why = Value::Null;
    while Instant::now() < deadline && !saved {
        let (code, body) = api.post(&format!("/api/deployments/{id}/save"), json!({}));
        saved = code == 200;
        why = body;
        if !saved {
            std::thread::sleep(Duration::from_secs(2));
        }
    }
    assert!(saved, "the first revision never saved: {why}\n{}", fleet.reconciler_log());

    assert!(
        wait_for_tag(&fleet, "alpha", Duration::from_secs(180)),
        "the first build never served — got {:?}\n--- node ---\n{}\n--- reconciler ---\n{}",
        tag_served(&fleet),
        fleet.node_log("n1"),
        fleet.reconciler_log()
    );
    println!("    v1 is serving `alpha`");

    // --- ship v2 over the top ------------------------------------------------
    // Same component id, different bytes. The platform mints a new digest, the
    // revision points at it, and the reconciler has to notice the app it is
    // running is not the app that is wanted.
    let digest_v1 = api.manifest_digest(&id);
    println!("    v1 manifest digest: {digest_v1}");
    assert!(matches!(api.upload("ver", beta.clone()), 200 | 201), "uploading beta failed");

    // Saving is RETRIED, because an upload clears the component's digest: the
    // bytes are staged, and the reconciler's push pass has to put them in the
    // object store and record the address before a revision can reference them
    // (ADR-0006). A single save fired immediately after the upload renders the
    // manifest against the digest that is still there — the old one — and
    // succeeds, which is a revision that ships nothing.
    let deadline = Instant::now() + Duration::from_secs(180);
    let mut served = false;
    while Instant::now() < deadline {
        let (code, _) = api.post(&format!("/api/deployments/{id}/save"), json!({}));
        let _ = code;
        if tag_served(&fleet).as_deref() == Some("beta") {
            served = true;
            break;
        }
        std::thread::sleep(Duration::from_secs(2));
    }

    assert!(
        served || wait_for_tag(&fleet, "beta", Duration::from_secs(60)),
        "the fleet kept serving the old build after a new one was uploaded and saved — \
         got {:?}. An agent that cannot replace what is running cannot ship anything.\n\
         digest before the upload: {digest_v1}\n\
         digest now:               {}\n\
         manifests: {}\n\
         --- node ---\n{}\n--- reconciler ---\n{}",
        tag_served(&fleet),
        api.manifest_digest(&id),
        api.get(&format!("/api/deployments/{id}/manifests")),
        fleet.node_log("n1"),
        fleet.reconciler_log()
    );
    println!("    v2 is serving `beta` — the recompiled artifact replaced the running one");

    // --- and it STAYS replaced ------------------------------------------------
    // A reconcile pass that flapped between digests would show up here: the old
    // one would come back within an interval or two.
    std::thread::sleep(Duration::from_secs(8));
    assert_eq!(
        tag_served(&fleet).as_deref(),
        Some("beta"),
        "the old build came back — the loop is flapping between two digests rather than \
         converging on the newest revision"
    );

    // The node ran both, which is what "replaced" means: not that the old one was
    // never there, but that it is not there now.
    let log = fleet.node_log("n1");
    assert!(
        log.contains("started ada/ver/"),
        "the node never reported starting anything:\n{log}"
    );
    println!("    and it stayed replaced");
}
