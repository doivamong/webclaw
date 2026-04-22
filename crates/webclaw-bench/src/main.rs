//! webclaw-bench — regression harness for webclaw extraction.
//!
//! Samples URLs from `benchmarks/targets_1000.txt` (pipe-delimited
//! `name|url|labels`), fetches (or loads from cache), extracts via
//! `webclaw-core`, and emits per-fixture + aggregate metrics as
//! JSON + human-readable report.
//!
//! Not a quality benchmark — heuristic pass/fail based on labels, not
//! ground-truth annotation. Intended for detecting regression after
//! cherry-picking upstream changes, not for positioning vs competitors.

// Averages computed from small counter values (<10^6) — precision loss
// from usize/u128 → f64 cast is not meaningful for this harness.
#![allow(clippy::cast_precision_loss)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use webclaw_core::{ExtractionResult, extract, is_probably_readable};
use webclaw_fetch::{FetchClient, FetchConfig};

#[derive(Parser, Debug)]
#[command(
    name = "webclaw-bench",
    about = "Extraction regression harness driven by benchmarks/targets_1000.txt"
)]
struct Args {
    /// Targets file, format: `name|url|comma,separated,labels` per line.
    #[arg(long, default_value = "benchmarks/targets_1000.txt")]
    targets: PathBuf,

    /// Sample size (0 = full corpus). Sampled from head of file, deterministic.
    #[arg(long, default_value_t = 20)]
    sample: usize,

    /// Filter: only targets whose labels contain ANY of the given comma-separated tokens.
    #[arg(long)]
    filter: Option<String>,

    /// Cache directory for fetched HTML. SHA256-keyed filenames.
    #[arg(long, default_value = "benchmarks/cache")]
    cache: PathBuf,

    /// Skip network fetch — only use cached HTML. Targets without cache are reported but skipped.
    #[arg(long)]
    from_cache: bool,

    /// Write baseline JSON to this path. Default: `benchmarks/baseline-<date>.json`.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Per-request timeout in seconds.
    #[arg(long, default_value_t = 15)]
    timeout: u64,
}

#[derive(Debug, Clone)]
struct Target {
    name: String,
    url: String,
    labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Outcome {
    name: String,
    url: String,
    labels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    word_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    markdown_bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extraction_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels_matched: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels_total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    readable: Option<bool>,
    from_cache: bool,
}

#[derive(Debug, Serialize)]
struct Report {
    timestamp: String,
    total_run: usize,
    successes: usize,
    failures: usize,
    readable_count: usize,
    avg_word_count: f64,
    avg_extraction_ms: f64,
    label_match_rate: f64,
    outcomes: Vec<Outcome>,
}

fn parse_targets(path: &Path) -> Result<Vec<Target>> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut out = Vec::new();
    for (lineno, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() != 3 {
            warn!(
                line = lineno + 1,
                "skipping malformed target (expected 3 `|` fields)"
            );
            continue;
        }
        out.push(Target {
            name: parts[0].trim().to_string(),
            url: parts[1].trim().to_string(),
            labels: parts[2]
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
        });
    }
    Ok(out)
}

fn filter_and_sample(targets: Vec<Target>, args: &Args) -> Vec<Target> {
    let filtered: Vec<Target> = if let Some(filter) = &args.filter {
        let wanted: Vec<String> = filter
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        targets
            .into_iter()
            .filter(|t| t.labels.iter().any(|l| wanted.contains(l)))
            .collect()
    } else {
        targets
    };

    if args.sample == 0 || args.sample >= filtered.len() {
        filtered
    } else {
        filtered.into_iter().take(args.sample).collect()
    }
}

fn cache_path(cache_dir: &Path, url: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let digest = hex::encode(hasher.finalize());
    cache_dir.join(format!("{}.html", &digest[..16]))
}

fn load_cache(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn save_cache(path: &Path, html: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let mut f = fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
    f.write_all(html.as_bytes())?;
    Ok(())
}

fn compute_label_match(plain_text: &str, labels: &[String]) -> (usize, usize) {
    let haystack = plain_text.to_lowercase();
    let total = labels.len();
    let matched = labels
        .iter()
        .filter(|l| l.len() >= 2 && haystack.contains(l.as_str()))
        .count();
    (matched, total)
}

async fn run_target(target: &Target, client: &FetchClient, args: &Args) -> Outcome {
    let cache_file = cache_path(&args.cache, &target.url);

    let (html, from_cache) = if let Some(cached) = load_cache(&cache_file) {
        (Ok(cached), true)
    } else if args.from_cache {
        return Outcome {
            name: target.name.clone(),
            url: target.url.clone(),
            labels: target.labels.clone(),
            error: Some("no cache and --from-cache set".into()),
            word_count: None,
            markdown_bytes: None,
            extraction_ms: None,
            labels_matched: None,
            labels_total: None,
            readable: None,
            from_cache: false,
        };
    } else {
        let fetched = client
            .fetch(&target.url)
            .await
            .map(|r| r.html)
            .map_err(|e| e.to_string());
        if let Ok(html) = &fetched
            && let Err(e) = save_cache(&cache_file, html)
        {
            warn!(url = %target.url, error = %e, "cache write failed");
        }
        (fetched, false)
    };

    match html {
        Err(e) => Outcome {
            name: target.name.clone(),
            url: target.url.clone(),
            labels: target.labels.clone(),
            error: Some(e),
            word_count: None,
            markdown_bytes: None,
            extraction_ms: None,
            labels_matched: None,
            labels_total: None,
            readable: None,
            from_cache,
        },
        Ok(html) => {
            let start = Instant::now();
            match extract(&html, Some(&target.url)) {
                Ok(result) => {
                    let readable = is_probably_readable(&result);
                    let ExtractionResult { content, .. } = result;
                    let elapsed = start.elapsed().as_millis();
                    let (matched, total) = compute_label_match(&content.plain_text, &target.labels);
                    Outcome {
                        name: target.name.clone(),
                        url: target.url.clone(),
                        labels: target.labels.clone(),
                        error: None,
                        word_count: Some(content.plain_text.split_whitespace().count()),
                        markdown_bytes: Some(content.markdown.len()),
                        extraction_ms: Some(elapsed),
                        labels_matched: Some(matched),
                        labels_total: Some(total),
                        readable: Some(readable),
                        from_cache,
                    }
                }
                Err(e) => Outcome {
                    name: target.name.clone(),
                    url: target.url.clone(),
                    labels: target.labels.clone(),
                    error: Some(format!("extract: {e}")),
                    word_count: None,
                    markdown_bytes: None,
                    extraction_ms: None,
                    labels_matched: None,
                    labels_total: None,
                    readable: Some(false),
                    from_cache,
                },
            }
        }
    }
}

fn aggregate(outcomes: Vec<Outcome>) -> Report {
    let timestamp = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    );
    let total_run = outcomes.len();
    let successes = outcomes.iter().filter(|o| o.error.is_none()).count();
    let failures = total_run - successes;
    let readable_count = outcomes.iter().filter(|o| o.readable == Some(true)).count();
    let avg_word_count = if successes == 0 {
        0.0
    } else {
        outcomes
            .iter()
            .filter_map(|o| o.word_count)
            .map(|w| w as f64)
            .sum::<f64>()
            / successes as f64
    };
    let avg_extraction_ms = if successes == 0 {
        0.0
    } else {
        outcomes
            .iter()
            .filter_map(|o| o.extraction_ms)
            .map(|m| m as f64)
            .sum::<f64>()
            / successes as f64
    };
    let (sum_matched, sum_total) = outcomes
        .iter()
        .filter_map(|o| o.labels_matched.zip(o.labels_total))
        .fold((0usize, 0usize), |(m, t), (mi, ti)| (m + mi, t + ti));
    let label_match_rate = if sum_total == 0 {
        0.0
    } else {
        sum_matched as f64 / sum_total as f64
    };

    Report {
        timestamp,
        total_run,
        successes,
        failures,
        readable_count,
        avg_word_count,
        avg_extraction_ms,
        label_match_rate,
        outcomes,
    }
}

fn print_report(report: &Report) {
    println!("---- per-fixture ----");
    for o in &report.outcomes {
        let tag = if o.error.is_some() { "FAIL" } else { "ok  " };
        let cache = if o.from_cache { "cache" } else { "net  " };
        let word = o.word_count.map_or_else(|| "-".into(), |w| w.to_string());
        let ms = o
            .extraction_ms
            .map_or_else(|| "-".into(), |m| m.to_string());
        let lbl = match (o.labels_matched, o.labels_total) {
            (Some(m), Some(t)) => format!("{m}/{t}"),
            _ => "-".into(),
        };
        let rd = match o.readable {
            Some(true) => "yes",
            Some(false) => "NO ",
            None => "-  ",
        };
        println!(
            "[{tag}] [{cache}] readable={rd} words={word:<6} ms={ms:<4} labels={lbl:<4} {}  {}",
            o.name, o.url
        );
        if let Some(e) = &o.error {
            println!("         error: {e}");
        }
    }
    println!("---- aggregate ----");
    println!(
        "total: {} (ok: {}, fail: {})",
        report.total_run, report.successes, report.failures
    );
    println!(
        "readable: {} / {} ({:.1}%)",
        report.readable_count,
        report.total_run,
        (report.readable_count as f64 / report.total_run.max(1) as f64) * 100.0
    );
    println!("avg word_count: {:.1}", report.avg_word_count);
    println!("avg extraction_ms: {:.1}", report.avg_extraction_ms);
    println!("label match rate: {:.1}%", report.label_match_rate * 100.0);
}

async fn async_main() -> Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .compact()
        .init();

    let targets = parse_targets(&args.targets)?;
    info!(count = targets.len(), "parsed targets");
    let selected = filter_and_sample(targets, &args);
    info!(count = selected.len(), "selected after filter/sample");

    fs::create_dir_all(&args.cache).ok();

    let client = FetchClient::new(FetchConfig {
        timeout: Duration::from_secs(args.timeout),
        ..FetchConfig::default()
    })?;

    let mut outcomes = Vec::with_capacity(selected.len());
    for target in &selected {
        let outcome = run_target(target, &client, &args).await;
        outcomes.push(outcome);
    }

    let report = aggregate(outcomes);
    print_report(&report);

    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(format!("benchmarks/baseline-{}.json", report.timestamp)));
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let json = serde_json::to_string_pretty(&report)?;
    fs::write(&output_path, json)?;
    println!("\nbaseline saved: {}", output_path.display());

    Ok(())
}

// Host the runtime on an explicitly-sized thread. wreq's BoringSSL handshake
// combined with async state machines from the per-target loop exceeds the
// Windows default 1 MB main-thread stack once multiple `client.fetch()` calls
// are in flight — see `benchmarks/README.md` troubleshooting.
fn main() -> Result<()> {
    std::thread::Builder::new()
        .name("webclaw-bench-main".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime")
                .block_on(async_main())
        })
        .expect("spawn bench main thread")
        .join()
        .expect("bench main thread panicked")
}
