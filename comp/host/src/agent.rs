//! The node agent: how one `comp-host` joins a lattice and does as it is told.
//!
//! Three jobs, and no more:
//!
//! * publish what is running here, as a full snapshot, on a timer;
//! * take `start`/`stop` commands and make them true; and
//! * fetch artifacts by digest.
//!
//! There is no scheduler here and no opinion about placement. The reconciler
//! decides; this obeys and reports. That split is the whole reason a node can be
//! added by installing a binary and joining a tailnet.
//!
//! **It keeps serving when the control plane is gone.** The instance table is
//! persisted on every accepted command and restored *before* NATS is contacted, so
//! a node that reboots during an outage comes back running what it was running. An
//! unreachable reconciler is not an instruction to stop.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::tenant::{instance_id, Limits, StartCommand};
use crate::{Instance, Instances, Routes};

/// Written by every host, read by the reconciler. The TTL on this bucket is what
/// makes a departed node disappear without anyone reaping anything.
const INVENTORY_BUCKET: &str = "comp-inventory";
/// Artifacts, keyed by their own sha256 (ADR-0024: the digest is the identity).
const ARTIFACT_BUCKET: &str = "comp-artifacts";

/// The capabilities this host can actually grant.
///
/// The successor to the renderer's `OPERATOR_BOUND`, and still the
/// highest-consequence list in the platform — except that it is now enforced by
/// the linker rather than by a renderer's omission. A component importing anything
/// not on it does not start.
/// VERSIONLESS on purpose, and this must stay in step with `manifest.rs`'s
/// `HostIface::family`. A node advertises a concrete version and a component
/// imports one; requiring the two strings to be equal made every deployment
/// permanently unschedulable the first time it was tried live, because the host
/// said `wasi:keyvalue/store@0.2.0-draft` and the manifest asked for
/// `wasi:keyvalue/store`.
// ponytail: family match; tighten to semver when two incompatible versions of one
// interface actually have to coexist on a node.
pub const HOST_IFACES: &[&str] = &[
    "wasi:http/incoming-handler",
    "wasi:http/outgoing-handler",
    "wasi:keyvalue/store",
    "wasi:keyvalue/atomics",
    "wasi:keyvalue/batch",
    "wasi:config/store",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RunningInstance {
    pub tenant: String,
    pub app: String,
    pub component: String,
    pub digest: String,
    pub count: u32,
}

#[derive(Serialize)]
struct Inventory<'a> {
    node: &'a str,
    labels: &'a BTreeMap<String, String>,
    host_ifaces: &'a [&'a str],
    kv_shared: bool,
    capacity: Capacity,
    instances: Vec<RunningInstance>,
}

#[derive(Serialize)]
struct Capacity {
    cpus: usize,
    instances: usize,
}

/// What a node needs to obey a command. Everything here is process-lifetime.
pub struct Agent {
    pub node: String,
    pub labels: BTreeMap<String, String>,
    pub lattice: String,
    pub engine: Arc<wasmtime::Engine>,
    pub kv: crate::Kv,
    pub cache_backing: crate::CacheBacking,
    pub instances: Instances,
    pub routes: Routes,
    pub limits: Limits,
    pub state_dir: PathBuf,
    pub heartbeat_secs: u64,
    /// Can every replica of an app see this node's store, wherever it runs?
    ///
    /// Advertised so the reconciler can refuse to spread a stateful app across
    /// nodes where it would silently diverge. `--kv sqlite`/`memory` say false.
    pub kv_shared: bool,
}

impl Agent {
    fn artifact_dir(&self) -> PathBuf {
        self.state_dir.join("artifacts")
    }

    fn ledger(&self) -> PathBuf {
        self.state_dir.join("instances.json")
    }

    /// Everything running here, for the inventory and for the ledger.
    fn snapshot(&self) -> Vec<RunningInstance> {
        self.instances
            .read()
            .unwrap()
            .values()
            .map(|i| RunningInstance {
                tenant: i.scope.tenant.clone(),
                app: i.scope.app.clone(),
                component: i.scope.component.clone(),
                digest: i.scope.digest.clone(),
                count: i.count,
            })
            .collect()
    }

    /// Persist what we were told to run, so a reboot is not a data-loss event for
    /// the fleet's desired state. Atomic rename: a half-written ledger read on the
    /// next boot would start a subset and look like a partial outage.
    fn persist(&self, commands: &BTreeMap<String, StartCommand>) {
        let tmp = self.ledger().with_extension("json.tmp");
        let Ok(bytes) = serde_json::to_vec_pretty(commands) else { return };
        if std::fs::write(&tmp, bytes).is_ok() {
            let _ = std::fs::rename(&tmp, self.ledger());
        }
    }

    fn load_ledger(&self) -> BTreeMap<String, StartCommand> {
        std::fs::read(self.ledger())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }
}

/// The commands this node has accepted, keyed by instance id. Kept beside the live
/// table because a `StartCommand` is what has to be replayed on boot — the
/// compiled instance cannot be.
type Ledger = Arc<std::sync::Mutex<BTreeMap<String, StartCommand>>>;

pub async fn run(agent: Arc<Agent>, nats_url: &str) -> Result<()> {
    std::fs::create_dir_all(agent.artifact_dir())
        .with_context(|| format!("creating {}", agent.artifact_dir().display()))?;

    let ledger: Ledger = Arc::new(std::sync::Mutex::new(agent.load_ledger()));

    // Restore BEFORE touching the network. This is the property that replaces the
    // operator: a node that reboots while the control plane is down comes back
    // serving, from its own disk, with no help from anyone.
    {
        let saved = ledger.lock().unwrap().clone();
        if !saved.is_empty() {
            eprintln!("comp-host: restoring {} instance(s) from the ledger", saved.len());
        }
        for (id, cmd) in saved {
            if let Err(e) = start(&agent, cmd, None).await {
                // A restore failure must not stop the others: one unreadable
                // artifact is not a reason to bring up nothing.
                eprintln!("comp-host: could not restore {id}: {e:#}");
            }
        }
    }

    let client = async_nats::connect(nats_url)
        .await
        .with_context(|| format!("connecting to NATS at {nats_url}"))?;
    let js = async_nats::jetstream::new(client.clone());

    let inventory = js
        .create_key_value(async_nats::jetstream::kv::Config {
            bucket: INVENTORY_BUCKET.into(),
            // Three missed beats. A flaky tailnet gets chances; a dead node does
            // not linger long enough to hold a replica hostage.
            max_age: Duration::from_secs(agent.heartbeat_secs * 3),
            ..Default::default()
        })
        .await
        .context("opening the inventory bucket")?;
    let artifacts = js
        .create_object_store(async_nats::jetstream::object_store::Config {
            bucket: ARTIFACT_BUCKET.into(),
            ..Default::default()
        })
        .await
        .context("opening the artifact store")?;

    // Heartbeat. Separate task so a slow command cannot make this node look dead
    // and get its work rescheduled underneath it.
    {
        let agent = agent.clone();
        let inventory = inventory.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_secs(agent.heartbeat_secs));
            loop {
                tick.tick().await;
                let inv = Inventory {
                    node: &agent.node,
                    labels: &agent.labels,
                    host_ifaces: HOST_IFACES,
                    kv_shared: agent.kv_shared,
                    capacity: Capacity {
                        cpus: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
                        instances: agent.instances.read().unwrap().len(),
                    },
                    instances: agent.snapshot(),
                };
                match serde_json::to_vec(&inv) {
                    Ok(bytes) => {
                        if let Err(e) = inventory.put(agent.node.as_str(), bytes.into()).await {
                            eprintln!("comp-host: heartbeat failed: {e}");
                        }
                    }
                    Err(e) => eprintln!("comp-host: could not serialise inventory: {e}"),
                }
            }
        });
    }

    let subject = format!("comp.{}.cmd.{}.>", agent.lattice, agent.node);
    let mut sub = client.subscribe(subject.clone()).await.context("subscribing to commands")?;
    eprintln!("comp-host: joined lattice {} as node {}", agent.lattice, agent.node);
    eprintln!("comp-host: listening on {subject}");

    while let Some(msg) = sub.next().await {
        let verb = msg.subject.rsplit('.').next().unwrap_or("").to_string();
        let reply = msg.reply.clone();
        let result = handle(&agent, &artifacts, &ledger, &verb, &msg.payload).await;

        // Acked only after the instance is built, so "started" means "will serve"
        // rather than "is downloading".
        let body = match &result {
            Ok(note) => serde_json::json!({ "ok": true, "note": note }),
            Err(e) => {
                eprintln!("comp-host: {verb} failed: {e:#}");
                serde_json::json!({ "error": format!("{e:#}") })
            }
        };
        if let Some(reply) = reply {
            let _ = client.publish(reply, body.to_string().into()).await;
        }
    }
    bail!("the command subscription ended — NATS closed the connection")
}

async fn handle(
    agent: &Arc<Agent>,
    artifacts: &async_nats::jetstream::object_store::ObjectStore,
    ledger: &Ledger,
    verb: &str,
    payload: &[u8],
) -> Result<String> {
    match verb {
        "start" => {
            let cmd: StartCommand =
                serde_json::from_slice(payload).context("unreadable start command")?;
            let id = instance_id(&cmd.tenant, &cmd.app, &cmd.component);
            start(agent, cmd.clone(), Some(artifacts)).await?;
            let saved = {
                let mut l = ledger.lock().unwrap();
                l.insert(id.clone(), cmd);
                l.clone()
            };
            agent.persist(&saved);
            Ok(format!("started {id}"))
        }
        "stop" => {
            #[derive(Deserialize)]
            struct Stop {
                tenant: String,
                app: String,
                component: String,
            }
            let s: Stop = serde_json::from_slice(payload).context("unreadable stop command")?;
            let id = instance_id(&s.tenant, &s.app, &s.component);
            // Shrink by the delta; remove only when nothing is left. A stop of 1
            // out of 3 must not take the whole instance down.
            // Stop means gone. Shrinking to a smaller non-zero count is a `start`
            // with a lower absolute count, so there is only one code path that
            // changes a replica count and only one that removes an instance.
            let removed = agent.instances.write().unwrap().remove(&id);
            if removed.is_some() {
                agent.routes.write().unwrap().retain(|_, v| *v != id);
            }
            let saved = {
                let mut l = ledger.lock().unwrap();
                l.remove(&id);
                l.clone()
            };
            agent.persist(&saved);
            Ok(if removed.is_some() {
                format!("stopped {id}")
            } else {
                // Not an error: the reconciler re-derives from inventory, so
                // stopping something already gone is a converged no-op.
                format!("{id} was not running")
            })
        }
        "drain" => {
            let ids: Vec<String> = agent.instances.read().unwrap().keys().cloned().collect();
            agent.instances.write().unwrap().clear();
            agent.routes.write().unwrap().clear();
            // The ledger is deliberately NOT cleared: a drain is an operator asking
            // this node to shed load now, not a decision that these apps should
            // never come back. The reconciler will place them elsewhere.
            Ok(format!("drained {} instance(s)", ids.len()))
        }
        other => bail!("unknown command {other:?}"),
    }
}

/// Build one instance and put it in the table.
///
/// `artifacts` is `None` during a ledger restore, where only the local cache may be
/// used — the whole point of that path is that it works with no network.
async fn start(
    agent: &Arc<Agent>,
    cmd: StartCommand,
    artifacts: Option<&async_nats::jetstream::object_store::ObjectStore>,
) -> Result<()> {
    let id = instance_id(&cmd.tenant, &cmd.app, &cmd.component);
    let ingress_host = cmd.ingress_host.clone();
    // A start command says how many this node should hold — an absolute count, not
    // a delta. Re-sending it is therefore a no-op, which matters because the
    // reconciler re-derives faster than this node heartbeats and will legitimately
    // repeat itself. (A delta here put six replicas of a two-replica app across two
    // machines on the first cross-machine run.)
    //
    // The clone-then-drop dance is not ceremony either. `if let Some(x) =
    // lock.read()…` holds the read guard for the whole block, so taking the write
    // lock inside it deadlocks — and because the heartbeat also reads this table,
    // the node then stops publishing inventory and gets its work rescheduled out
    // from under it. Also measured, not theorised.
    let resized = {
        let table = agent.instances.read().unwrap();
        table
            .get(&id)
            .filter(|e| e.scope.digest == cmd.digest && e.count != cmd.count.max(1))
            .map(|e| {
                Arc::new(Instance {
                    scope: e.scope.clone(),
                    pre: e.pre.clone(),
                    count: cmd.count.max(1),
                })
            })
    };
    if let Some(resized) = resized {
        let n = resized.count;
        agent.instances.write().unwrap().insert(id.clone(), resized);
        eprintln!("comp-host: {id} now holds {n} replica(s)");
        return Ok(());
    }
    // Already exactly as asked: say so and touch nothing.
    if agent.instances.read().unwrap().get(&id).is_some_and(|e| e.scope.digest == cmd.digest) {
        return Ok(());
    }

    // Omission fails closed. A component importing something this host cannot
    // grant is refused HERE, at start, rather than trapping on its first request
    // in front of a user.
    for need in &cmd.host_needs {
        if !HOST_IFACES.contains(&need.as_str()) {
            bail!("{id} imports {need}, which this host cannot grant");
        }
    }

    let path = fetch_artifact(agent, &cmd.digest, artifacts).await?;
    let count = cmd.count.max(1);
    let scope = Arc::new(cmd.into_scope(&agent.limits));

    // Compilation is slow and blocking; a start command must not stall the
    // heartbeat behind it.
    let engine = agent.engine.clone();
    let component = tokio::task::spawn_blocking(move || {
        wasmtime::component::Component::from_file(&engine, &path)
    })
    .await
    .context("compile task")?
    .map_err(|e| anyhow::anyhow!("compiling the artifact for {id}: {e}"))?;

    let linker = crate::build_linker(&agent.engine)?;
    let pre = wasmtime_wasi_http::p2::bindings::ProxyPre::new(linker.instantiate_pre(&component)?)
        .map_err(|e| {
            anyhow::anyhow!(
                "{id} does not export wasi:http/incoming-handler ({e}). Runtime-linking a \
                 plug component in-process is not built yet — deploy it fused for now"
            )
        })?;

    agent
        .instances
        .write()
        .unwrap()
        .insert(id.clone(), Arc::new(Instance { scope: scope.clone(), pre, count }));
    if let Some(host) = ingress_host {
        agent.routes.write().unwrap().insert(host.to_ascii_lowercase(), id.clone());
    }
    eprintln!("comp-host: started {id} ({})", scope.digest);
    Ok(())
}

/// Local cache first, object store second, and the digest is checked either way.
///
/// The object store is not a trust boundary — the digest is. Anything that does not
/// hash to the name it was fetched under is discarded rather than compiled.
async fn fetch_artifact(
    agent: &Arc<Agent>,
    digest: &str,
    artifacts: Option<&async_nats::jetstream::object_store::ObjectStore>,
) -> Result<PathBuf> {
    let short = digest.trim_start_matches("sha256:");
    if short.is_empty() || !short.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("{digest:?} is not a sha256 digest");
    }
    let path = agent.artifact_dir().join(format!("{short}.wasm"));
    if path.exists() {
        return Ok(path);
    }
    let Some(store) = artifacts else {
        bail!("{digest} is not in the local cache and there is no store to fetch it from");
    };

    let mut object = store
        .get(digest)
        .await
        .with_context(|| format!("fetching {digest} from the artifact store"))?;
    let mut bytes = Vec::new();
    tokio::io::AsyncReadExt::read_to_end(&mut object, &mut bytes)
        .await
        .context("reading the artifact")?;

    let got = sha256_hex(&bytes);
    if got != short {
        bail!("artifact {digest} hashes to sha256:{got} — refusing to compile it");
    }
    write_atomic(&path, &bytes)?;
    Ok(path)
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("wasm.tmp");
    std::fs::write(&tmp, bytes).with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| format!("renaming into {}", path.display()))?;
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_digest_that_is_not_a_digest_is_refused() {
        // The artifact path is a filename built from this string. A digest with a
        // slash in it would be a directory traversal into whatever the host can
        // write, so it is validated as hex before it is ever joined to a path.
        for bad in ["", "sha256:", "sha256:../../etc/passwd", "sha256:zz", "latest", "sha256:ab/cd"]
        {
            let short = bad.trim_start_matches("sha256:");
            let ok = !short.is_empty() && short.chars().all(|c| c.is_ascii_hexdigit());
            assert!(!ok, "{bad:?} must not pass the digest check");
        }
        let good = "sha256:deadbeefcafe";
        let short = good.trim_start_matches("sha256:");
        assert!(!short.is_empty() && short.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn the_host_interface_list_is_what_the_linker_actually_provides() {
        // This list is the successor to the renderer's OPERATOR_BOUND and it is
        // still the highest-consequence list in the platform: anything on it is
        // something a tenant's component may import. Adding a line here without
        // adding it to `build_linker` promises something that then fails at start.
        assert!(HOST_IFACES.iter().all(|i| i.contains(':') && i.contains('/')));
        // Versionless, matching what a manifest stamps into `host_needs`. If these
        // ever carry a version again, every deployment becomes unschedulable.
        assert!(!HOST_IFACES.iter().any(|i| i.contains('@')));
        // wasmcloud:messaging stays off it. Raw subject publish is the one
        // capability that reaches around the host's naming, which would break the
        // boundary for every NATS-backed thing at once.
        assert!(!HOST_IFACES.iter().any(|i| i.starts_with("wasmcloud:messaging")));
    }

    #[test]
    fn sha256_matches_the_reconcilers() {
        // Both sides content-address independently; if these ever disagree, every
        // artifact fetch fails its integrity check for a reason nobody would guess.
        assert_eq!(
            sha256_hex(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
