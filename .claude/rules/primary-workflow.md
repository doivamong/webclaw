# Primary Workflow — webclaw

<!-- Adapted from ITF_APP primary-workflow.md (user's own work) + inspired by claudekit-engineer (commercial, paraphrased). Rust/webclaw context. -->

**Rule này mô tả workflow TỔNG QUAN** cho Claude Code khi làm việc trong webclaw. Chi tiết từng bước xem SKILL.md tương ứng trong `.claude/skills/`.

## Nguyên tắc nền tảng

- **YAGNI / KISS / DRY** (xem [development-rules.md](development-rules.md))
- **Plan trước, code sau** — HARD GATE trong `wc-cook`
- **Verify bằng evidence, không đoán** — HARD GATE trong `wc-debug-map`
- **Crate boundary tuyệt đối** — WASM-safe core, primp isolation (xem [crate-boundaries.md](crate-boundaries.md))
- **Domain rules luôn thắng workflow rules** khi mâu thuẫn

## 7 task type → skill chain

### 1. Implement feature mới (full workflow)

```
wc-predict           (5 personas — feature lớn / risky)
  ↓
wc-scenario          (10 dimensions edge cases — nếu stateful)
  ↓
wc-cook              (7 steps: scout → plan → review gate → implement → test → review → finalize)
  ↓ (orchestrate bên trong)
wc-arch-guard + wc-config-guard + wc-mcp-guard + wc-output-guard
  ↓
wc-review-v2         (3-stage: spec → quality → adversarial)
  ↓
wc-pre-commit
```

### 2. Debug bug có cấu trúc

```
wc-debug-map         (6 steps: scout → diagnose → assess → fix → verify+prevent → finalize)
  ↓ (domain-specific khi user nói rõ)
wc-arch-guard | wc-config-guard | wc-mcp-guard | wc-bot-detection-audit
  ↓
wc-pre-commit
```

### 3. Review code đã viết

```
wc-review-v2         (3-stage: spec → R1-R8 quality → adversarial red-team)
  ↓ (nếu changes required)
Fix + wc-review-v2 re-run
  ↓
wc-pre-commit
```

**Scope gate:** Stage 3 adversarial skip nếu ≤2 files VÀ ≤30 dòng VÀ không chạm `crates/webclaw-core/` hoặc `crates/webclaw-mcp/`.

### 4. Nghiên cứu / so sánh phương án

```
wc-research-guide    (4 steps: scope → explore → compare → recommend)
  ↓ (optional, quyết định kỹ thuật quan trọng)
wc-predict           (5 personas stress-test)
  ↓
Document decision + rationale
```

### 5. Tối ưu hiệu suất

```
wc-optimize          (MEASURE trước khi fix — cargo bench + flamegraph + hyperfine)
  ↓
wc-review-v2 + wc-pre-commit
```

### 6. Refactor lớn / tách crate / rename module

```
wc-graph             (cargo-modules, cargo depgraph — blast radius map)
  ↓
wc-arch-guard        (dependency direction, WASM-safe)
  ↓
wc-output-guard      (completeness khi tạo file mới)
  ↓
wc-pre-commit
```

### 7. Port code từ repo GitHub / commercial boilerplate

```
wc-github-ref        (3 modes: lookup / study / adoption)
  ↓ (adoption path only — guarded gate)
License check → wc-arch-guard → wc-research-guide → wc-predict
  ↓
wc-cook (mode --fast vì research đã có)
  ↓
wc-pre-commit
```

## Mode selection cho wc-cook

| Mode | Research | Test | Review Gate | Dùng khi |
|------|----------|------|-------------|----------|
| `--interactive` (default) | ✓ | ✓ | User approve mỗi step | Feature phức tạp, cần user input |
| `--fast` | ✗ | ✓ | User approve mỗi step | Đã có research từ predict/github-ref |
| `--auto` | ✓ | ✓ | Auto nếu score ≥9.5 | Task rõ, tin auto |

## Blocking gates (không bao giờ skip)

1. **Post-Plan:** Plan được approve TRƯỚC implement (HARD GATE `wc-cook`)
2. **Diagnose:** Root cause xác định TRƯỚC fix (HARD GATE `wc-debug-map`)
3. **Testing:** `cargo test --workspace` pass (unless `--no-test` mode explicit)
4. **Pre-commit:** `wc-pre-commit` 10-item checklist
5. **Hard rules:** [crate-boundaries.md](crate-boundaries.md) — WASM-safe core, primp isolation

## Required tools per phase

| Phase | Tool/Skill | Mandatory? |
|-------|-----------|-----------|
| Research (cho feature lớn) | wc-predict, wc-scenario | Optional (fast mode skip) |
| Plan | wc-cook Step 2 | BẮT BUỘC (HARD GATE) |
| Implement | wc-cook Step 4 (+guards invoke bên trong) | BẮT BUỘC |
| Test | `cargo test --workspace` (0 failures unless no-test mode) | BẮT BUỘC |
| Review | wc-review-v2 (3-stage) | BẮT BUỘC nếu thay đổi ≥1 file |
| Finalize | wc-pre-commit | **LUÔN LUÔN** last step |

## Priority conflict resolution

Khi nhiều skill cùng match:
- **Workflow (Tier 0)** > **Guard (Tier 1)** > **Feature (Tier 2)** > **Audit (Tier 3)**
- Guard skills không bypass được (arch-guard, config-guard, mcp-guard, wasm boundary hook)
- **Domain rule thắng workflow rule** khi conflict (xem [development-rules.md](development-rules.md))
- Chi tiết conflict resolution: xem `using-skills` meta-skill (auto-inject qua SessionStart hook)

## KHÔNG làm trong bất kỳ workflow nào

- KHÔNG viết code trước plan (HARD GATE wc-cook)
- KHÔNG fix bug trước khi root cause xác định (HARD GATE wc-debug-map)
- KHÔNG import `tokio` / `reqwest` / `wreq` / `std::fs` / `std::net` vào `crates/webclaw-core/**` (WASM-safe)
- KHÔNG thêm `[patch.crates-io]` ở crate-level Cargo.toml (chỉ workspace root)
- KHÔNG dùng `wreq` trong `crates/webclaw-llm/` (plain `reqwest` only, LLM APIs không cần TLS impersonation)
- KHÔNG leak qwen3 `<think>` tag xuống consumer (strip 2 tầng: provider + consumer)
- KHÔNG skip pre-commit check trước commit
- KHÔNG bypass safety hooks bằng `--no-verify` trừ khi user request rõ

## Hook integration (auto-fire, xem Phase 5)

| Event | Hook | Action |
|-------|------|--------|
| SessionStart | `session_start_inject.py` | Inject using-skills meta-skill vào context |
| PreToolUse (Write/Edit) | `secret_scanner.py` | Block ghi API key literal |
| PostToolUse (Edit/Write on `crates/webclaw-core/**`) | `wasm_boundary_check.py` | Block import network/fs crate |
| PostToolUse (Edit/Write on `*.rs`) | `cargo_fmt_check.py` | Warn nếu `cargo fmt --check` fail |

**Fail-open:** mọi hook có `@hook_main(name)` decorator → crash log JSONL, exit 0. Kill switch: `WEBCLAW_HOOK_LOGGING=0`.
