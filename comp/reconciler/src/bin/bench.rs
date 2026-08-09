//! The reading half of the benchmarks: parse a result, print one honest line.
//!
//! These were eight small Python scripts that each grew their own idea of what
//! "failed" means. Two of them silently reported nothing for the exact interval being
//! measured, and one counted every 5xx as a shed when most were the ingress saying it
//! had no route at all — mistakes that survived because each script was the only
//! reader of its own output.
//!
//! One binary, one set of definitions. The scripts still orchestrate processes and
//! ssh, which is what shell is good at; nothing interprets a number in bash.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;

#[derive(Parser)]
#[command(name = "comp-bench", about = "Read benchmark output and say what it means")]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// One line from an `oha --output-format json` result.
    Summarise {
        path: std::path::PathBuf,
        #[arg(default_value = "")]
        label: String,
    },
    /// Median phase costs from a host's own `started … in N us (…)` lines.
    Coldstart { log: std::path::PathBuf },
    /// Total replicas the fleet is running, from an inventory dump on stdin.
    Replicas,
    /// Which node holds what, read straight from the lattice.
    Inventory {
        #[arg(long, default_value = "nats://127.0.0.1:4232")]
        nats_url: String,
        #[arg(long, default_value = "default")]
        lattice: String,
    },
}

fn main() -> Result<()> {
    match Args::parse().cmd {
        Cmd::Summarise { path, label } => summarise(&path, &label),
        Cmd::Coldstart { log } => coldstart(&log),
        Cmd::Replicas => replicas(),
        Cmd::Inventory { nats_url, lattice } => inventory(&nats_url, &lattice),
    }
}

/// The fleet as the reconciler sees it.
///
/// Read through the lattice crate rather than by shelling out to `nats kv get`, which
/// also retires a footgun that cost two runs: `--raw` writes no trailing newline, so
/// several nodes concatenated into one invalid line and the sampler reported nothing
/// for exactly the window being measured.
fn inventory(nats_url: &str, lattice: &str) -> Result<()> {
    use comp_lattice::nats::NatsLattice;
    use comp_lattice::Inventory as _;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let l = NatsLattice::connect(nats_url, lattice, std::time::Duration::from_secs(15)).await?;
        let mut total = 0u64;
        let mut rows: Vec<String> = Vec::new();
        for e in l.read_all().await? {
            let Ok(v) = serde_json::from_slice::<Value>(&e.value) else { continue };
            let mut held: Vec<String> = Vec::new();
            if let Some(items) = v["instances"].as_array() {
                for i in items {
                    let n = i["count"].as_u64().unwrap_or(0);
                    total += n;
                    held.push(format!(
                        "{}/{} x{n}",
                        i["tenant"].as_str().unwrap_or("?"),
                        i["app"].as_str().unwrap_or("?")
                    ));
                }
            }
            rows.push(format!(
                "    {:10} ({:>2} cpu) {}",
                v["node"].as_str().unwrap_or("?"),
                v["capacity"]["cpus"].as_u64().unwrap_or(0),
                if held.is_empty() { "-".to_string() } else { held.join(", ") }
            ));
        }
        rows.sort();
        for r in rows {
            println!("{r}");
        }
        println!("    total {total} replica(s)");
        Ok::<_, anyhow::Error>(())
    })
}

fn summarise(path: &std::path::Path, label: &str) -> Result<()> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let d: Value = serde_json::from_str(&text).with_context(|| format!("{}", path.display()))?;
    let s = &d["summary"];
    let p = &d["latencyPercentiles"];
    let codes: BTreeMap<String, u64> = serde_json::from_value(
        d["statusCodeDistribution"].clone(),
    )
    .unwrap_or_default();
    let errs: BTreeMap<String, u64> =
        serde_json::from_value(d["errorDistribution"].clone()).unwrap_or_default();

    let ok: u64 = codes.iter().filter(|(k, _)| k.starts_with('2')).map(|(_, v)| v).sum();
    let non2xx: u64 = codes.iter().filter(|(k, _)| !k.starts_with('2')).map(|(_, v)| v).sum();
    // oha counts requests still in flight when the clock stops as errors. With 200
    // connections that is 200 every run, and calling it "failed" overstates every
    // result by exactly the connection count.
    let aborted: u64 = errs.iter().filter(|(k, _)| k.contains("deadline")).map(|(_, v)| v).sum();
    let transport: u64 = errs.values().sum::<u64>() - aborted;

    println!(
        "  {label:14} {:8.0} rps   p50 {:7.1} ms   p99 {:8.1} ms   {ok} ok / {non2xx} non-2xx / {transport} failed / {aborted} in flight at end",
        s["requestsPerSec"].as_f64().unwrap_or(0.0),
        1000.0 * p["p50"].as_f64().unwrap_or(0.0),
        1000.0 * p["p99"].as_f64().unwrap_or(0.0),
    );
    if non2xx > 0 && ok == 0 {
        // The failure that produced ADR-0036's correction: 102k rps that were all
        // ingress 503s, reported as "100% success" because oha counts completions.
        println!("  {:14} NO 2xx AT ALL — this measured an error path: {codes:?}", "");
    }
    Ok(())
}

fn median(mut v: Vec<u64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_unstable();
    v[v.len() / 2] as f64
}

fn coldstart(log: &std::path::Path) -> Result<()> {
    let text = std::fs::read_to_string(log).with_context(|| format!("{}", log.display()))?;
    let (mut cold, mut warm) = (Vec::new(), Vec::new());
    for line in text.lines() {
        let Some(rest) = line.split(" in ").nth(1) else { continue };
        let Some((total, tail)) = rest.split_once(" us (") else { continue };
        let Ok(total) = total.trim().parse::<u64>() else { continue };
        let nums: Vec<u64> = tail
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();
        if nums.len() < 3 {
            continue;
        }
        let row = (total, nums[0], nums[1], nums[2]);
        if tail.contains("cache-load") { warm.push(row) } else { cold.push(row) }
    }
    for (rows, what) in [(&cold, "cold: wasmtime compiles"), (&warm, "warm: loaded from cache")] {
        if rows.is_empty() {
            continue;
        }
        println!("=== {what}: {} start(s) ===", rows.len());
        for (i, name) in ["total", "fetch", "build", "link"].iter().enumerate() {
            let vals: Vec<u64> = rows
                .iter()
                .map(|r| match i {
                    0 => r.0,
                    1 => r.1,
                    2 => r.2,
                    _ => r.3,
                })
                .collect();
            println!(
                "  {name:8} median {:8.2} ms   min {:7.2} ms   max {:7.2} ms",
                median(vals.clone()) / 1000.0,
                *vals.iter().min().unwrap() as f64 / 1000.0,
                *vals.iter().max().unwrap() as f64 / 1000.0
            );
        }
    }
    if !cold.is_empty() && !warm.is_empty() {
        let (a, b) = (median(cold.iter().map(|r| r.0).collect()),
                      median(warm.iter().map(|r| r.0).collect()));
        println!(
            "\n  {:.1} ms -> {:.2} ms, a {:.0}x cut ({:.1}% of the start removed).",
            a / 1000.0,
            b / 1000.0,
            a / b.max(1.0),
            100.0 * (a - b) / a.max(1.0)
        );
    }
    Ok(())
}

fn replicas() -> Result<()> {
    // One JSON inventory entry per line, as `nats kv get … --raw` produces once each
    // is newline-terminated. `--raw` writes NO trailing newline, which silently
    // concatenated several nodes into one invalid line and reported zero for exactly
    // the interval being measured — hence reading line by line and skipping junk
    // rather than parsing the whole stream as one document.
    let mut total = 0u64;
    for line in std::io::stdin().lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
        total += v["instances"]
            .as_array()
            .map(|a| a.iter().filter_map(|i| i["count"].as_u64()).sum::<u64>())
            .unwrap_or(0);
    }
    println!("{total}");
    Ok(())
}
