---
name: wc-output-guard
origin: adapted-from-itf
inspired-by: itf-output-guard (2026-04-22, paraphrased)
user-invocable: false
description: >
  GUARD — BẮT BUỘC khi generate code dài (>200 dòng) hoặc viết toàn bộ file.
  Đảm bảo code hoàn chỉnh — không truncate, không placeholder, không "...".
  USE WHEN: viết toàn bộ file Rust, thêm module mới, tạo crate mới,
  refactor lớn, viết integration test đầy đủ.
  Ví dụ trigger: "viết toàn bộ", "generate", "tạo file", "thêm module",
  "refactor", "viết lại", "add feature", "write complete", "entire file".
  DO NOT TRIGGER when: output <50 dòng hoặc sửa nhỏ (<10 dòng) trong file hiện có.
triggers:
  - "generate"
  - "viết code"
  - "viết toàn bộ"
  - "tạo file mới"
  - "đầy đủ code"
  - "full code"
  - "entire file"
  - "write complete"
---

Announce: "Đang dùng wc-output-guard — đảm bảo output hoàn chỉnh."

# webclaw Output Guard

**Nguyên tắc:** Treat every code generation as production-critical. A partial output is a broken output.

---

## Banned Patterns — Rust (CRITICAL)

```rust
// ... rest of function
// ... rest of impl
// ... rest of module
// TODO: implement
// (other methods here)
// same pattern as above
todo!()         // chỉ OK trong branch chưa scope
unimplemented!()// chỉ OK trong trait default method đang thiết kế
panic!("not implemented")  // banned trong lib code
return Err(unimplemented!())
```

## Banned Patterns — Cargo.toml (CRITICAL)

```toml
# ... more deps
# TODO: pin version
version = "*"       # banned, phải semver range cụ thể
version = "1"       # quá lỏng, ít nhất "1.0" hoặc "1.0.x"
[dependencies]
# add more as needed
```

## Banned Patterns — SKILL.md / Markdown (IMPORTANT)

```
<!-- ... more examples ... -->
<!-- TODO: add section -->
[placeholder]
(more details coming)
```

## Banned Phrases (IMPORTANT)

```
"for brevity"
"etc.", "and so on", "..."
"same as before" / "same as above"
"similar to the previous"
"you can add more"
"I'll leave this part to you"
"omitted for space"
"abbreviated"
"rest is similar"
```

---

## Long File Protocol (CRITICAL — file > 200 dòng)

Khi generate file Rust dài (ví dụ `extractor.rs` ~1486 dòng, `markdown.rs` ~1431 dòng, `brand.rs` ~1340 dòng, `main.rs` CLI ~2372 dòng):

1. **Viết hoàn chỉnh** đến clean breakpoint:
   - End of `fn` (function)
   - End of `impl` block
   - End of `#[cfg(test)] mod tests { ... }` block
   - End of top-level `mod` declaration

2. **Đánh dấu điểm dừng:**

   ```rust
   // === TIẾP THEO: [tên function/impl section tiếp theo] ===
   ```

3. **Thông báo rõ ràng:**

   > "Đã viết đến `impl Extractor::score()`. Gõ **'tiếp'** để tiếp tục phần còn lại (impl Extractor::filter + mod tests)."

4. **Không được:** compress, skip, rút gọn, tóm tắt nội dung còn lại

**Clean breakpoints hợp lệ:**

- End of Rust `fn`
- End of `impl T for U { ... }`
- End of `trait Foo { ... }`
- End of `#[cfg(test)] mod tests { ... }`
- End of top-level `mod foo { ... }` declaration
- End of `pub use ...;` block

---

## Pre-output Verification Checklist (IMPORTANT)

Trước khi submit response, kiểm tra:

```
[ ] Không có banned Rust pattern nào
[ ] Không có banned Cargo.toml pattern
[ ] Không có banned phrase trong prose/comment
[ ] Tất cả requested items đều có mặt và đầy đủ
[ ] Code blocks chứa runnable implementation, không phải pseudocode
[ ] `pub` items có doc comment (`///`)
[ ] `use` statements sorted + deduplicated
[ ] Nếu file dài → đã dùng Long File Protocol với clean breakpoint
[ ] Test (nếu có) cover happy path + edge case
```

---

## Inline Verification (IMPORTANT — file >100 dòng)

Sau mỗi Write/Edit file >100 dòng, TRƯỚC KHI tiếp tục task:

1. **Read lại file** — xác nhận nội dung đúng
2. **Grep kiểm tra banned patterns:**

   ```bash
   grep -cn "// \.\.\." <file>      # expected 0
   grep -cn "todo!()" <file>        # expected 0 trong lib code
   grep -cn "unimplemented!()" <file> # expected 0 trong lib code
   grep -cn "placeholder" <file>    # expected 0
   ```

3. **Đếm dòng** — so với estimate ban đầu (sai lệch >20% → review)
4. **`cargo check -p webclaw-<crate>`** — verify syntax + type

Nếu BẤT KỲ check fail → fix NGAY, trước khi viết file tiếp.

---

**Violating the letter of these rules is violating the spirit of these rules.**

---

## Failure Modes Registry (tích lũy theo thời gian)

> Mỗi khi gây failure mới liên quan skill này, thêm entry cuối. KHÔNG xóa entry cũ.

| # | Trigger | Symptom | Fix | Severity | First seen |
|---|---------|---------|-----|----------|------------|
| G1 | Generate `extractor.rs` 800 dòng truncate giữa `impl` | `cargo check` fail với "expected }" — user copy-paste thiếu closing brace | Long File Protocol — break ở end-of-impl + user gõ "tiếp" | CRITICAL | 2026-04-22 |
| G2 | `todo!()` trong `fn scrape()` (public API) | Runtime panic khi user gọi tool → MCP response 500 | `todo!()` chỉ OK trong private branch chưa scope; public API phải có impl | HIGH | 2026-04-22 |
| G3 | `// ... rest of selectors` trong extractor `once_cell::Lazy` init | Missing selector → extraction quality drop 30% trên benchmark corpus | Viết full selector list, không rút gọn | CRITICAL | 2026-04-22 |
| G4 | Long File Protocol dùng xong, user chưa gõ "tiếp" | Claude assume done, file còn dở → `cargo build` fail sau commit | KHÔNG commit/finalize cho đến khi user confirm "tiếp" xong toàn bộ | HIGH | 2026-04-22 |
| G5 | Compress `Cargo.toml` dependency list "for brevity" | Missing transitive dep → build fail trên clean machine | Liệt kê đầy đủ deps, không dùng "for brevity" | HIGH | 2026-04-22 |

**Khi thêm entry**: ghi rõ Trigger, Symptom, Fix, Severity, ngày.
