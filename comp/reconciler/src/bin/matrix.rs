//! The benchmark matrix: every dimension crossed, and load held rather than spiked.
//!
//! Every number this project has published so far moved one axis for fifteen or
//! twenty seconds. That is enough to catch a throughput difference and exactly wrong
//! for the questions now being asked — what an idle app costs, whether sharing helps
//! at scale, whether anything leaks. A 20-second spike cannot see drift, and a
//! one-axis run cannot see an interaction.
//!
//! So: a cell per combination, load held for `--seconds`, and memory sampled
//! throughout rather than read once at the end.
//!
//! ```
//! comp-matrix --apps 1,8,32 --seconds 120
//! comp-matrix --apps 32 --seconds 600 --only distinct   # one long cell
//! ```

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use comp_reconciler::fleet::{bin_path, Fleet};

#[derive(Parser)]
#[command(name = "comp-matrix", about = "Cross every dimension, hold the load")]
struct Args {
    /// App counts to try, e.g. `1,8,32`.
    #[arg(long, default_value = "1,8,32", value_delimiter = ',')]
    apps: Vec<usize>,

    /// How long to hold load in each cell. Short runs cannot see drift.
    #[arg(long, default_value = "120")]
    seconds: u64,

    /// Concurrent connections during the load phase.
    #[arg(long, default_value = "64")]
    conns: usize,

    /// `same` (every app on one digest) or `distinct` (a digest each). Both by default.
    #[arg(long)]
    only: Option<String>,

    /// Also run each cell with wasmtime's pooling allocator.
    #[arg(long)]
    with_pool: bool,

    /// Nodes per fleet.
    #[arg(long, default_value = "1")]
    nodes: u16,
}

#[derive(Debug)]
struct Cell {
    apps: usize,
    digests: &'static str,
    pool: bool,
    idle_mib: f64,
    loaded_mib: f64,
    drift_mib: f64,
    per_app_mib: f64,
    rps: f64,
    p50_ms: f64,
    p99_ms: f64,
    shared: usize,
    loaded_from_disk: usize,
    compiled: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
    let base = root.join("components/target/gate_domain.composed.wasm");
    anyhow::ensure!(base.exists(), "missing {} — just compose-gate", base.display());
    let wasm = std::fs::read(&base)?;

    let modes: Vec<&'static str> = match args.only.as_deref() {
        Some("same") => vec!["same"],
        Some("distinct") => vec!["distinct"],
        None => vec!["same", "distinct"],
        Some(other) => anyhow::bail!("--only takes `same` or `distinct`, not {other:?}"),
    };
    let pools: Vec<bool> = if args.with_pool { vec![false, true] } else { vec![false] };

    let total = args.apps.len() * modes.len() * pools.len();
    eprintln!(
        "comp-matrix: {total} cell(s), {}s of load each — about {} minutes\n",
        args.seconds,
        (total as u64 * (args.seconds + 40)) / 60
    );

    let mut cells = Vec::new();
    for &apps in &args.apps {
        for &digests in &modes {
            for &pool in &pools {
                eprintln!("  running apps={apps} digests={digests} pool={pool} …");
                cells.push(run_cell(&root, &wasm, apps, digests, pool, &args)?);
            }
        }
    }
    report(&cells, &args);
    Ok(())
}

/// N specs, and N artifacts when each app should have its own digest.
///
/// A distinct digest is the SAME component with a custom section appended — a wasm
/// custom section is inert, so behaviour is identical and only the content address
/// differs. Using genuinely different components instead would confound the memory
/// question with "these do different work".
fn write_inputs(
    dir: &std::path::Path,
    wasm: &[u8],
    apps: usize,
    distinct: bool,
) -> Result<Vec<(String, std::path::PathBuf)>> {
    std::fs::create_dir_all(dir.join("specs"))?;
    std::fs::create_dir_all(dir.join("art"))?;
    let mut artifacts = Vec::new();
    for i in 0..apps {
        let component = if distinct { format!("gate{i}") } else { "gate".to_string() };
        std::fs::write(
            dir.join("specs").join(format!("app{i}.yaml")),
            format!(
                "version: comp/v1\napp: app{i}\ntenant: t{i}\nstrategy: fused\n\
                 components:\n  - id: {component}\n\
                 ingress:\n  host: app{i}.matrix.test\n  component: {component}\n"
            ),
        )?;
        if distinct || i == 0 {
            let path = dir.join("art").join(format!("{component}.wasm"));
            let mut bytes = wasm.to_vec();
            if distinct {
                bytes.extend_from_slice(&custom_section(&format!("comp-matrix-{i}")));
            }
            std::fs::write(&path, &bytes)?;
            artifacts.push((component, path));
        }
    }
    Ok(artifacts)
}

/// A wasm custom section: id 0, then a LEB128 length, then a named payload. Ignored
/// by every runtime, which is the point — it changes the digest and nothing else.
fn custom_section(name: &str) -> Vec<u8> {
    let mut body = Vec::new();
    leb128(name.len() as u64, &mut body);
    body.extend_from_slice(name.as_bytes());
    let mut out = vec![0u8];
    leb128(body.len() as u64, &mut out);
    out.extend_from_slice(&body);
    out
}

fn leb128(mut v: u64, out: &mut Vec<u8>) {
    loop {
        let mut b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
        }
        out.push(b);
        if v == 0 {
            return;
        }
    }
}

fn rss_mib(pid: u32) -> f64 {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok();
    out.and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map(|kb| kb / 1024.0)
        .unwrap_or(0.0)
}

fn run_cell(
    root: &std::path::Path,
    wasm: &[u8],
    apps: usize,
    digests: &'static str,
    pool: bool,
    args: &Args,
) -> Result<Cell> {
    let dir = tempfile::tempdir()?;
    let artifacts = write_inputs(dir.path(), wasm, apps, digests == "distinct")?;
    let arts: Vec<String> = artifacts
        .iter()
        .map(|(id, p)| format!("{id}={}", p.display()))
        .collect();

    let lattice = format!("mx{apps}{digests}{}", u8::from(pool));
    let fleet = Fleet::start_bench(
        &lattice,
        dir.path().join("specs").to_str().unwrap(),
        &arts,
        args.nodes,
        pool,
    );

    // Placed, then settled: an RSS read while instances are still arriving measures
    // the arrival, not the resting cost.
    let deadline = Instant::now() + Duration::from_secs(120);
    while Instant::now() < deadline && fleet.started_count() < apps {
        std::thread::sleep(Duration::from_millis(500));
    }
    std::thread::sleep(Duration::from_secs(5));
    let pid = fleet.host_pid(1).context("the host is not running")?;
    let idle_mib = rss_mib(pid);

    // --- sustained load, memory sampled throughout ---
    let stop = Arc::new(AtomicBool::new(false));
    let (ok, lat_sum, lat_max) =
        (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let mut workers = Vec::new();
    for w in 0..args.conns {
        let (stop, ok, lat_sum, lat_max) =
            (stop.clone(), ok.clone(), lat_sum.clone(), lat_max.clone());
        let url = format!("http://127.0.0.1:{}/api/ratelimit", fleet.ingress_port);
        // Spread the load across every app, not just the first: one hot app and 31
        // idle ones is a different measurement from 32 apps in use.
        let host = format!("app{}.matrix.test", w % apps);
        workers.push(std::thread::spawn(move || {
            let client = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap();
            while !stop.load(Ordering::Relaxed) {
                let t = Instant::now();
                let sent = client
                    .post(&url)
                    .header("host", &host)
                    .json(&serde_json::json!({
                        "key": "m", "capacity": 100_000_000u64, "refill": 100_000_000u64
                    }))
                    .send();
                if sent.map(|r| r.status().is_success()).unwrap_or(false) {
                    let us = t.elapsed().as_micros() as u64;
                    ok.fetch_add(1, Ordering::Relaxed);
                    lat_sum.fetch_add(us, Ordering::Relaxed);
                    lat_max.fetch_max(us, Ordering::Relaxed);
                }
            }
        }));
    }

    // Let it reach steady state before the drift window opens, or the ramp counts as
    // a leak.
    std::thread::sleep(Duration::from_secs(10));
    let settled = rss_mib(pid);
    let started = Instant::now();
    let mut peak = settled;
    while started.elapsed() < Duration::from_secs(args.seconds.saturating_sub(10)) {
        std::thread::sleep(Duration::from_secs(5));
        peak = peak.max(rss_mib(pid));
    }
    let loaded_mib = rss_mib(pid);
    stop.store(true, Ordering::Relaxed);
    for w in workers {
        let _ = w.join();
    }

    let served = ok.load(Ordering::Relaxed);
    let elapsed = args.seconds as f64;
    let (shared, loaded_from_disk, compiled) = fleet.module_arrivals(1);
    Ok(Cell {
        apps,
        digests,
        pool,
        idle_mib,
        loaded_mib,
        // Growth AFTER steady state. Under a constant arrival rate this should be
        // flat; anything else is the thing a 20-second spike cannot see.
        drift_mib: loaded_mib - settled,
        per_app_mib: (idle_mib - 12.0) / apps as f64,
        rps: served as f64 / elapsed,
        p50_ms: if served > 0 {
            lat_sum.load(Ordering::Relaxed) as f64 / served as f64 / 1000.0
        } else {
            0.0
        },
        p99_ms: lat_max.load(Ordering::Relaxed) as f64 / 1000.0,
        shared,
        loaded_from_disk,
        compiled,
    })
}

fn report(cells: &[Cell], args: &Args) {
    println!("\n=== matrix: {}s of load per cell, {} conns ===\n", args.seconds, args.conns);
    println!(
        "  {:>5} {:>9} {:>5} │ {:>8} {:>9} {:>8} │ {:>9} {:>8} {:>8} │ {:>6} {:>5} {:>4}",
        "apps", "digests", "pool", "idle MiB", "loaded", "per-app", "rps", "mean ms", "max ms",
        "shared", "disk", "cc"
    );
    for c in cells {
        println!(
            "  {:>5} {:>9} {:>5} │ {:>8.1} {:>9.1} {:>8.2} │ {:>9.0} {:>8.2} {:>8.1} │ {:>6} {:>5} {:>4}",
            c.apps,
            c.digests,
            if c.pool { "on" } else { "off" },
            c.idle_mib,
            c.loaded_mib,
            c.per_app_mib,
            c.rps,
            c.p50_ms,
            c.p99_ms,
            c.shared,
            c.loaded_from_disk,
            c.compiled
        );
    }

    println!("\n  drift after steady state (a leak shows up here, not in a spike):");
    for c in cells {
        let verdict = if c.drift_mib > 5.0 {
            "  <- GROWING"
        } else if c.drift_mib > 1.0 {
            "  <- watch"
        } else {
            ""
        };
        println!(
            "    apps={:<4} digests={:<9} pool={:<4} {:+.2} MiB over {}s{verdict}",
            c.apps,
            c.digests,
            if c.pool { "on" } else { "off" },
            c.drift_mib,
            args.seconds - 10
        );
    }

    // The comparison the matrix exists for: same-digest against distinct at equal
    // app counts. One number per pair, rather than two runs to eyeball.
    println!("\n  what sharing a digest saves, at equal app count:");
    for c in cells.iter().filter(|c| c.digests == "same") {
        if let Some(d) =
            cells.iter().find(|o| o.digests == "distinct" && o.apps == c.apps && o.pool == c.pool)
        {
            let saved = d.idle_mib - c.idle_mib;
            println!(
                "    {:>3} apps: {:>6.1} MiB shared vs {:>6.1} distinct — {:+.1} MiB ({:.0}% less)",
                c.apps,
                c.idle_mib,
                d.idle_mib,
                -saved,
                100.0 * saved / d.idle_mib.max(1.0)
            );
        }
    }
    let _ = bin_path("comp-host");
}
