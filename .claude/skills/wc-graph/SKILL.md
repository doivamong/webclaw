---
name: wc-graph
origin: adapted-from-itf
inspired-by: itf-graph-assist (2026-04-22, paraphrased — cargo-modules/depgraph)
description: >
  USE WHEN user hỏi về cấu trúc crate, module tree, callers của symbol,
  dependency graph, hoặc tìm function trong codebase.
  BẮT BUỘC dùng khi user nói:
  "X.rs có gì?", "liệt kê hàm trong X", "tìm fn [name]",
  "ai gọi hàm X", "dependency graph", "crate tree",
  "file này có gì", "cấu trúc module", "hàm dài", "hàm quá dài",
  "rebuild graph", "thống kê codebase".
  DO NOT TRIGGER when: "code thừa"/"dead code" (→ wc-code-audit),
  "chậm"/"tối ưu" (→ wc-optimize), "bug"/"panic" (→ wc-debug-map).
argument-hint: "[file-hoặc-tên-fn]"
allowed-tools: Read, Grep, Glob, Bash(cargo *)
triggers:
  - "cấu trúc file"
  - "liệt kê hàm"
  - "file này có gì"
  - "route nào trong"
  - "tìm fn"
  - "tên này dùng chưa"
  - "hàm dài"
  - "hàm quá dài"
  - "ai gọi"
  - "callers"
  - "dependency graph"
  - "crate tree"
  - "module tree"
  - "thống kê codebase"
  - "rebuild graph"
  - "graph"
---

Announce: "Đang dùng wc-graph — scan cấu trúc + dependency."

# webclaw Graph Assist

Tool set để hiểu cấu trúc code, tìm function, xem dependency.

## Required Tools

```bash
cargo install cargo-modules   # module tree visualization
cargo install cargo-depgraph  # crate dependency graph
# rust-analyzer trong VS Code/IntelliJ cho LSP queries
```

## Common Tasks

### 1. Module tree của 1 crate

```bash
cargo modules generate tree -p webclaw-core --with-types
# Output: tree of modules, types, functions
```

Output example:
```
crate webclaw-core
├── mod brand
│   ├── fn extract_brand
│   └── struct Brand
├── mod extractor
│   ├── fn score
│   └── fn extract
├── mod markdown
...
```

### 2. Crate dependency graph (workspace)

```bash
cargo depgraph --all-deps | dot -Tsvg -o deps.svg
# Hoặc simpler:
cargo tree --workspace --depth 1
```

Check dependency direction match `.claude/rules/crate-boundaries.md` (cli → mcp → fetch/llm/pdf → core).

### 3. Tìm callers của function

```bash
# Grep-based
grep -rn "function_name(" crates/*/src/

# Với context 3 dòng
grep -rn -B1 -A3 "function_name(" crates/*/src/

# rust-analyzer LSP: "Find all references" (IDE)
```

⚠ **Lazy import blind spot**: Rust không có `from x import y` dynamic. Mọi call phải có `use` statement trong scope hoặc full path. Grep `function_name(` đủ chính xác.

### 4. List `pub` API của 1 crate

```bash
cargo modules generate tree -p webclaw-<crate> --with-types | grep "pub "

# Hoặc:
cargo doc -p webclaw-<crate> --no-deps --open
# Browse HTML docs, `pub` items được list
```

### 5. Hàm dài / Impl dài

```bash
# awk để count LOC per fn
awk '/^(pub )?(async )?fn / { fn=$0; count=0 }
     /^}$/ { if (count > 80) print FILENAME":"NR" — "count" lines — "fn; count=0 }
     { count++ }' crates/*/src/*.rs
```

Threshold reference (development-rules.md):
- Function >80 dòng → nên tách
- Impl block >300 dòng → nên split sub-module

### 6. Thống kê codebase

```bash
# LOC per crate
for c in crates/webclaw-*; do
  echo "=== $(basename $c) ==="
  find $c/src -name "*.rs" | xargs wc -l | tail -1
done

# Số fn public per crate
for c in crates/webclaw-*; do
  count=$(grep -rn "^pub fn\|^pub async fn" $c/src/ | wc -l)
  echo "$(basename $c): $count pub fn"
done

# Complexity: function >80 dòng
awk '/^(pub )?(async )?fn / { fn=$0; count=0; file=FILENAME }
     /^}$/ { if (count > 80) print file": "fn; count=0 }
     { count++ }' crates/*/src/*.rs | sort | uniq -c | sort -rn
```

### 7. Check symbol trùng tên

```bash
# Function trùng tên giữa crate
grep -rn "^\(pub \)\?\(async \)\?fn extract_" crates/*/src/
# Kiểm tra namespace conflict trước đặt tên mới
```

## Integration với webclaw

| Query | Tool | When |
|-------|------|------|
| "hàm `score` ở đâu" | `grep -rn "fn score" crates/webclaw-core/src/` | Locate function |
| "ai gọi `data_island`" | `grep -rn "data_island" crates/*/src/` | Callers impact |
| "extractor.rs có gì" | `cargo modules generate tree -p webclaw-core` + grep `mod extractor` | Module summary |
| "crate tree" | `cargo depgraph --all-deps` hoặc `cargo tree --workspace --depth 1` | Dependency view |
| "hàm nào >80 dòng" | awk script trên | Complexity check |

## Lazy Import Blind Spot

Rust **không có** lazy import runtime — mọi `use` statement resolve compile-time. Khác với Python.

Implication: `grep 'fn_name('` đủ chính xác tìm call site. Không có "gọi qua reflection" nguy hiểm như Python.

**Exception**: macro expansion có thể hide call (vd: `#[derive(Debug)]` generate impl). Khi audit impact, check:
- `cargo expand -p <crate> --lib <module>` xem expanded code
- Grep pattern macro sử dụng: `derive(Debug\|Clone\|Serialize)`

## DO NOT Use wc-graph for

- Code quality / dead code → `wc-code-audit`
- Performance bottleneck → `wc-optimize`
- Bug root cause → `wc-debug-map`

wc-graph CHỈ là STRUCTURE / RELATIONSHIP query, không phải QUALITY check.

## Output Format

```
## Graph Query: [question]

### Tool used
`cargo modules generate tree -p webclaw-core`

### Result
[Tree output hoặc grep result]

### Interpretation
- Found N matches
- Notable: [specific file:line]
- Related callers: [list]

### Next (nếu relevant)
- Long function >80 LOC → consider split (wc-code-audit sau)
- Cross-crate tight coupling → wc-arch-guard review
```
