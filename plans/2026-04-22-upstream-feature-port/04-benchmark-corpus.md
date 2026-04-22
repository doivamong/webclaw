# 04 — Benchmark Corpus từ `targets_1000.txt`

**Date**: 2026-04-22
**Type**: Benchmark infrastructure (supersedes study-followup `02-benchmark-harness.md`)
**Status**: Ready, 1 session
**Crate(s) affected**: NEW `webclaw-bench` (workspace member), `benchmarks/`
**Context**: Upstream có `targets_1000.txt` — 1000 URL labeled (Nike, StockX, Amazon, etc.) format `name|url|labels`. Perfect seed cho corpus thay vì tự collect 18 fixture manual.

## Executive Summary

**SUPERSEDES** `plans/2026-04-22-study-followup/02-benchmark-harness.md`. Dùng upstream's 1000-URL corpus làm seed. Fork build harness đọc, sample subset, cache HTML fixtures, compute regression baseline. Ship faster + richer diversity than 18 manual fixtures.

## Why supersede study-followup 02

| Aspect | Study-followup 02 | This plan |
|---|---|---|
| Corpus source | Manual collect 18 fixture | Upstream 1000 URL seed + sample strategy |
| Coverage | 13 readability + 5 bot | 1000 real-world diverse (e-commerce, social, docs) |
| Diversity | Hand-pick biased | Upstream curated (Nike, Amazon, StockX, ...) |
| Fixture effort | 1 session collect | Zero — targets ready |
| Ground-truth effort | 1 session annotate 18 | Optional: sample 30-50 for annotation, rest use heuristic pass/fail |

## Requirements

- [ ] `webclaw-bench` crate compiles, ship with binary
- [ ] Port + commit `targets_1000.txt` vào `benchmarks/`
- [ ] Harness accepts sampling strategy (full / N-sample / category-filter)
- [ ] Output baseline JSON for regression check
- [ ] `wc-extraction-bench` skill có thể invoke harness

## Phases

### Phase 1 — Port targets_1000.txt (30 min)

**Source**: `research/github_0xMassi_webclaw/targets_1000.txt`
**Target**: `D:/webclaw/benchmarks/targets_1000.txt`

**Format** (confirmed từ sample):
```
Nike PDP|https://www.nike.com/t/air-force-1-07-mens-shoes-jBrhbr/CW2288-111|nike,air force,cart
Nike Women|https://www.nike.com/w/womens-running-shoes-37v7jz5e1x6|nike,women,running
StockX PDP|https://stockx.com/nike-dunk-low-retro-white-black-2021|stockx,dunk,bid
Amazon US|https://www.amazon.com/dp/B0CX23V2ZK|amazon,price,cart
```

Fields: `name | URL | labels (comma-separated)`

**Action**:
```bash
cp research/github_0xMassi_webclaw/targets_1000.txt benchmarks/
```

**Attribution**: Upstream AGPL-3.0 = fork AGPL-3.0, add entry to `ATTRIBUTIONS.md`:
```markdown
## targets_1000.txt (benchmark corpus seed)

- **Source**: https://github.com/0xMassi/webclaw/blob/main/targets_1000.txt (AGPL-3.0)
- **Used in**: `benchmarks/targets_1000.txt`
- **Adaptations**: None (verbatim port). May filter/subset for local runs.
```

**Acceptance**:
- [ ] `benchmarks/targets_1000.txt` exists
- [ ] `wc -l benchmarks/targets_1000.txt` == 1000
- [ ] ATTRIBUTIONS.md entry

**Commit**: `feat(bench): port targets_1000.txt corpus seed from upstream`

### Phase 2 — Create `webclaw-bench` crate (1-2h)

**Files**:
- `D:/webclaw/crates/webclaw-bench/Cargo.toml` (new)
- `D:/webclaw/crates/webclaw-bench/src/main.rs` (new)
- `D:/webclaw/crates/webclaw-bench/src/fixture.rs` (new)
- `D:/webclaw/crates/webclaw-bench/src/metrics.rs` (new)

**Cargo.toml**:
```toml
[package]
name = "webclaw-bench"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "webclaw-bench"
path = "src/main.rs"

[dependencies]
webclaw-core = { workspace = true }
webclaw-fetch = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
clap = { workspace = true }
tokio = { workspace = true }
anyhow = "1"
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[lints]
workspace = true
```

**main.rs skeleton**:
```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "webclaw-bench")]
struct Args {
    /// Targets file (pipe-delimited: name|url|labels)
    #[arg(long, default_value = "benchmarks/targets_1000.txt")]
    targets: PathBuf,
    /// Sample size (0 = full corpus)
    #[arg(long, default_value_t = 50)]
    sample: usize,
    /// Label filter (e.g., "amazon,stockx")
    #[arg(long)]
    filter: Option<String>,
    /// Output baseline JSON
    #[arg(long)]
    output: Option<PathBuf>,
    /// Cache fetched HTML to dir
    #[arg(long)]
    cache: Option<PathBuf>,
    /// Use cached HTML instead of refetch
    #[arg(long)]
    from_cache: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // 1. Parse targets
    // 2. Sample or filter
    // 3. Fetch (with cache) or load cache
    // 4. Extract per URL
    // 5. Compute metrics per fixture
    // 6. Aggregate + write JSON
    Ok(())
}
```

**Acceptance**:
- [ ] `cargo build -p webclaw-bench` pass
- [ ] `cargo run -p webclaw-bench -- --help` show CLI

### Phase 3 — Metrics + harness logic (1 session)

**Metrics per fixture** (no ground-truth required for most):
- `word_count` (extracted text length)
- `score` (from `score_node`)
- `title_length`
- `link_count`
- `is_probably_readable` (heuristic pass/fail)
- `extraction_time_ms`
- `label_matched` (fixture label trong extracted content — "cart", "price", etc.)

**No ground-truth required heuristics**:
- Fixture labeled `cart`, `price` → extracted text SHOULD contain "cart" / "price" keyword → pass
- Fixture labeled `article`, `blog` → extracted word_count SHOULD > 200 → pass
- Fixture labeled `login`, `404` → `is_probably_readable` SHOULD return false → pass

**Aggregate output**:
```json
{
  "timestamp": "2026-04-22T15:30:00Z",
  "sample_size": 50,
  "total_targets": 1000,
  "metrics": {
    "pass_rate": 0.84,
    "avg_extraction_time_ms": 12.3,
    "avg_word_count": 840,
    "avg_score": 145.2
  },
  "per_category": {
    "cart": { "count": 12, "pass_rate": 0.92 },
    "article": { "count": 8, "pass_rate": 0.88 },
    ...
  },
  "per_fixture": [
    { "name": "Nike PDP", "score": 187, "pass": true, ... },
    ...
  ]
}
```

**Acceptance**:
- [ ] Harness chạy trên 50-sample <60s (with cache)
- [ ] JSON output parse valid
- [ ] `wc-extraction-bench` có command reference

### Phase 4 — Cache HTML + reproducibility (1h)

**Cache directory**: `benchmarks/cache/` (gitignore) hoặc `benchmarks/fixtures/` (commit, smaller sample 30-50).

**Strategy**:
- Default: `--cache benchmarks/cache/` (gitignored, local only)
- For CI: `--from-cache benchmarks/fixtures/` (30 hand-picked fixtures committed)
- Hybrid: 30 fixtures committed làm regression gate, 1000 full corpus cho manual run

**Acceptance**:
- [ ] `--cache` populate dir với HTML per URL
- [ ] `--from-cache` skip fetch, re-extract from cached HTML
- [ ] `benchmarks/fixtures/` commit 30 subset cho CI

### Phase 5 — Update `benchmarks/README.md` (fix marketing fiction)

**Problem (từ audit trước)**: `benchmarks/README.md` claim `cargo run -p webclaw-bench` + 94.2% accuracy + 50 fixtures — ALL fake pre-plan.

**Action**: Rewrite to match reality:
```markdown
# Benchmarks

`webclaw-bench` crate + 1000-URL corpus (ported from upstream `targets_1000.txt`).

## Quick run

```bash
# Default 50-sample with cache
cargo run --release -p webclaw-bench -- --cache benchmarks/cache/

# Full 1000-corpus (cached subsequent runs)
cargo run --release -p webclaw-bench -- --sample 0

# Filter by label
cargo run --release -p webclaw-bench -- --filter "amazon,stockx"

# CI regression check (30-fixture subset)
cargo run --release -p webclaw-bench -- --from-cache benchmarks/fixtures/
```

## Corpus

1000 URLs labeled (Nike, Amazon, StockX, Medium, Wikipedia, etc.) trong `targets_1000.txt`. Format: `name|url|labels`.

Subset of 30 URLs với cached HTML committed trong `benchmarks/fixtures/` for reproducible CI gate.

## Metrics

Heuristic pass/fail không cần manual annotation:
- Label `cart`/`price` → extracted text must contain keyword
- Label `article`/`blog` → word_count ≥ 200
- Label `login`/`404` → `is_probably_readable` returns false

Baseline snapshots: `benchmarks/baseline-YYYY-MM-DD.json`.
```

**Remove fake claims**: "94.2% accuracy", "Mozilla Readability 87.3%", "trafilatura 80.6%" — không có data backing. Revisit sau khi real harness run stable 2-3 tuần.

**Acceptance**:
- [ ] README.md không claim metric không có evidence
- [ ] Command examples chạy thực (`cargo run -p webclaw-bench` works)

**Commit**: `docs(bench): rewrite README.md to reflect actual harness capability`

## Architecture

```mermaid
flowchart LR
  TARGETS[targets_1000.txt<br/>1000 URLs labeled] --> PARSER[Parse name|url|labels]
  PARSER --> SAMPLE[Sample N or filter label]
  SAMPLE --> CACHE_CHECK{From cache?}
  CACHE_CHECK -->|Yes| LOAD[Load HTML from benchmarks/cache/]
  CACHE_CHECK -->|No| FETCH[webclaw-fetch fetch URL]
  FETCH --> STORE[Cache HTML to disk]
  LOAD --> EXTRACT[webclaw-core extract_content]
  STORE --> EXTRACT
  EXTRACT --> METRICS[Compute heuristic metrics]
  METRICS --> AGG[Aggregate per-label + overall]
  AGG --> JSON[benchmarks/baseline-YYYY-MM-DD.json]
  AGG --> STDOUT[Human-readable report]
```

## Risk Assessment

| Risk | Impact | Mitigation |
|---|---|---|
| 1000 fetch chậm + rate limit | High | Cache mandatory, default `--sample 50` |
| Some URL dead link | Med | Graceful skip, log warning, continue |
| Heuristic metric false positive | Med | 30-fixture subset có manual ground-truth cho high-confidence gate |
| Fetch rate limit trigger bot protection | Med | Spread across browser profile (Random mode), respect robots.txt optional |

## Integration với study-followup `03-readability-research.md`

- **A1 threshold calibration**: Phase này produce score distribution → use for A1
- **B1 gap analysis**: Compare webclaw vs dom_smoothie CLI trên same 50-sample → gap evidence
- **C1 flamegraph**: `cargo flamegraph --release -p webclaw-cli -- --urls-file benchmarks/targets_1000.txt --sample 20` trực tiếp dùng corpus

## Acceptance (overall)

- [ ] `targets_1000.txt` committed (attribution OK)
- [ ] `webclaw-bench` crate build + run
- [ ] Default 50-sample run <60s (from cache after first run)
- [ ] `benchmarks/README.md` hết marketing fiction
- [ ] Study-followup plan 03 có corpus unblock A1/B1/C1

## Next

- Replace references trong `plans/2026-04-22-study-followup/02-benchmark-harness.md` → mark as SUPERSEDED, delete file hoặc point to this plan
- Enable `wc-extraction-bench` skill với real harness
