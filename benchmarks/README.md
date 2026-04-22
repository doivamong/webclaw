# Benchmarks

Regression harness for webclaw extraction. Not a competitive benchmark,
not ground-truth annotated — designed to detect extraction regression
after cherry-picking upstream changes or modifying core logic.

## Quick run

```bash
# Default 20-sample with network fetch + cache
cargo run --release -p webclaw-bench

# Filter to e-commerce targets (labels contain any of nike,amazon,stockx)
cargo run --release -p webclaw-bench -- --filter nike,amazon,stockx

# 50-sample, cache-only (after a previous run populated cache)
cargo run --release -p webclaw-bench -- --sample 50 --from-cache

# Full 1000-target corpus (with cache, prepare for 10+ minutes first run)
cargo run --release -p webclaw-bench -- --sample 0
```

## Corpus

`targets_1000.txt` — 1000 real-world URLs labeled as `name|url|labels`,
ported from upstream `0xMassi/webclaw` v0.4.0. Covers Nike, Amazon,
StockX, Shopify, news, docs, SPAs, social. See `ATTRIBUTIONS.md`.

## Metrics

Per-target:
- `word_count` — plain text word count after extraction
- `markdown_bytes` — size of rendered markdown
- `extraction_ms` — extraction wall time (DOM parse + score + render)
- `labels_matched / labels_total` — count of labels (lowercased) that
  appear as substring in extracted plain text. Heuristic signal, not
  strict correctness. Higher is better.

Aggregate:
- successes / failures
- avg word_count
- avg extraction_ms
- label match rate (sum matched / sum total across all targets)

## Cache

HTML cached in `benchmarks/cache/<sha256-prefix>.html` (gitignored).
Second run on same targets hits cache, no network. Cache invalidation:
delete the dir, or specific files by sha prefix.

## Output

JSON baseline written to `benchmarks/baseline-<unix-ts>.json` (gitignored).
Each file is a timestamped snapshot — keep locally for comparison, not
committed. Baseline format:

```json
{
  "timestamp": "1776866389",
  "total_run": 20, "successes": 19, "failures": 1,
  "avg_word_count": 284.3, "avg_extraction_ms": 42.1,
  "label_match_rate": 0.58,
  "outcomes": [ { "name": "...", "url": "...", ... } ]
}
```

## Workflow for regression check

```bash
# 1. Before change: record baseline
git checkout main
cargo run --release -p webclaw-bench -- --sample 100 --output /tmp/before.json

# 2. Apply change (edit, cherry-pick, merge)

# 3. After change: run same sample from cache
cargo run --release -p webclaw-bench -- --sample 100 --from-cache --output /tmp/after.json

# 4. Diff aggregates
jq '{avg_word_count, avg_extraction_ms, label_match_rate}' /tmp/before.json /tmp/after.json
```

If `label_match_rate` drops >5% or `avg_word_count` drops >10%, the change
regressed extraction quality — investigate per-fixture outcomes.

## Not ground-truth

This harness does NOT verify extraction correctness against human
annotation. `labels_matched` is heuristic — a Nike PDP with labels
`nike,air force,cart` scoring 2/3 could mean "cart" is genuinely missing
from the extracted content, OR the checkout UI text was noise-stripped
correctly. Use as directional signal, not absolute quality score.
