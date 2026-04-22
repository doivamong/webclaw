# Attributions

webclaw fork (`doivamong/webclaw`) is licensed under AGPL-3.0. Portions of this
codebase are adapted from third-party projects under permissive licenses. This
file tracks attribution required by those licenses.

## Relationship to upstream `0xMassi/webclaw`

This repository is a fork of `github.com/0xMassi/webclaw` (also AGPL-3.0).
Upstream CHANGELOG at `research/github_0xMassi_webclaw/CHANGELOG.md` tracks
releases v0.1.0 through v0.4.0. Upstream AGPL-3.0 = fork AGPL-3.0, no separate
attribution block required for direct merges — git history carries authorship.

## Ported / adapted code

<!-- Add entry per port:
## <function / pattern name>

- **Source**: https://github.com/<owner>/<repo> (<license>)
- **Original**: <URL to specific file/line>
- **Used in**: `crates/webclaw-<crate>/src/<file>.rs` (<function name>)
- **Adaptations**: <brief note>
-->

## benchmark corpus seed (targets_1000.txt)

- **Source**: https://github.com/0xMassi/webclaw (AGPL-3.0), `targets_1000.txt` at root
- **Used in**: `benchmarks/targets_1000.txt`
- **Adaptations**: None (verbatim copy). 1000 URLs in `name|url|labels` format used as benchmark corpus seed — fork can sample/filter subsets, harness is fork-specific.
- **Why attribution tracked explicitly** even though upstream is same AGPL-3.0: the file contains curated third-party URL selection (Nike, Amazon, StockX, etc.) — upstream's editorial effort, acknowledge.

## CJK punctuation heuristic (score_node)

- **Source**: https://github.com/spider-rs/readability (MIT)
- **Original**: `src/scorer.rs:21` — `PUNCTUATIONS_REGEX`
- **Used in**: `crates/webclaw-core/src/extractor.rs` (`CJK_PUNCTUATIONS` static + bonus logic in `score_node`)
- **Adaptations**: Regex simplified to CJK-only (`[、。，．！？]`). Latin punctuation already handled by text-length heuristics. Bonus capped at +10 so CJK content competes with English without overwhelming other signals.

## Reference-only studies

Study reports at `research/github_*/_wc_ref_meta.md` are not code ports, only
pattern learning. No attribution required since no source was copied.
