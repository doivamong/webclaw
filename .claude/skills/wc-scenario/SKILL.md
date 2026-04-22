---
name: wc-scenario
origin: adapted-from-itf
inspired-by: itf-scenario (2026-04-22, paraphrased — 10 dimensions webclaw)
description: >
  ANALYSIS — Phân tích edge case qua 10 dimensions cho webclaw.
  Dùng trước implement hoặc viết test để phát hiện issue sớm.
  USE WHEN: trước khi implement feature stateful/multi-input, trước khi viết test,
  risk assessment, API design review.
  Ví dụ: "edge case", "scenario", "kịch bản", "test case", "boundary",
  "trường hợp đặc biệt", "sinh test".
  DO NOT TRIGGER when: task đơn giản 1 dòng, config change thuần,
  code đã stable không thay đổi gần đây.
triggers:
  - "edge case"
  - "scenario"
  - "kịch bản"
  - "test case"
  - "boundary"
  - "trường hợp đặc biệt"
  - "sinh test"
---

Announce: "Đang dùng wc-scenario — phân tích edge cases qua 10 dimensions."

## 10 Dimensions — webclaw (CRITICAL)

Không phải 10 đều áp dụng mọi feature. Xác định dimension relevant trước, skip kèm lý do.

| # | Dimension | webclaw-specific Focus |
|---|-----------|------------------------|
| 1 | **Locale / Language** | English, Vietnamese (diacritics), Japanese (Shift_JIS), Chinese (GBK/GB2312), Arabic (RTL), mixed script, language detection fail |
| 2 | **Bot Protection** | Cloudflare Turnstile (full + widget embedded), CF challenge page ("Just a moment"), DataDome, AWS WAF, hCaptcha, Akamai, PerimeterX — detection threshold edge |
| 3 | **Content-Type** | `text/html`, `application/xhtml+xml`, `application/pdf`, `text/plain`, `application/json`, SPA-with-`Content-Type: text/html`, missing Content-Type, wrong declared vs actual |
| 4 | **Encoding** | UTF-8 (default), UTF-16 with BOM, GBK, Shift_JIS, ISO-8859-1, Windows-1252, wrong charset declaration, mixed encoding trong single document |
| 5 | **Page Size** | Empty page (`<html></html>`), tiny (<1KB challenge page), medium (50-500KB typical), large (>10MB e-commerce catalog), extreme (>100MB single page), streaming timeout |
| 6 | **SPA Hydration** | React/Vue/Svelte SSR, client-only rendering (JS required), `__NEXT_DATA__` JSON data island, Contentful CMS JSON, Vite/Webpack bundle, hydration mismatch |
| 7 | **Redirect Chain** | 0 redirects (direct), 1-3 redirects (normal), 5+ (suspicious loop), cross-origin redirect, cookie-based redirect, JS-meta redirect (`<meta http-equiv="refresh">`), HTTP/HTTPS scheme change |
| 8 | **Sitemap / Discovery** | `sitemap.xml` (standard), `sitemap_index.xml` (nested), compressed `.xml.gz`, JSON sitemap, `robots.txt` disallow, no sitemap, 403 on sitemap |
| 9 | **Network / Timeout** | 200 OK, 301/302 redirect, 403 Forbidden (bot block), 404 Not Found, 429 Rate Limit, 500 Server Error, 503 Service Unavailable, connection timeout, DNS failure, TLS handshake fail |
| 10 | **Proxy / Auth** | Direct (no proxy), rotating proxy pool (healthy), proxy dead (connection refused), proxy auth required, proxy returns wrong content (MITM), Bearer token expired, Basic auth required |

## Workflow (IMPORTANT)

1. **Đọc** target file(s) hoặc feature description
2. **Filter dimensions** — đánh dấu 10 dimension nào áp dụng; skip kèm lý do rõ ràng
3. **Generate 3-5 scenario** mỗi dimension relevant
4. **Phân loại severity** — Critical / High / Medium / Low
5. **Output** bảng kết quả

## Severity Criteria (IMPORTANT)

| Level | Nghĩa webclaw-specific |
|-------|------------------------|
| **Critical** | Data loss, panic lib code, WASM boundary violate, secret leak, MCP client crash, bot detection false negative (real challenge missed → user thấy garbage HTML) |
| **High** | Feature broken cho subset input (locale/encoding miss), extraction quality regression >10% trên corpus, retry loop infinite |
| **Medium** | UX kém (warning không hiển thị), extraction quality regression 5-10%, cache stale, rate limit not respected |
| **Low** | Visual glitch nhỏ trong markdown output, non-blocking log warning, format sai nhưng data đúng semantic |

## Output Format (IMPORTANT)

```
## Scenario Report: [target]

Dimensions analyzed: [list]
Dimensions skipped: [list + lý do]

| # | Dimension | Scenario | Severity | Expected Behavior |
|---|-----------|----------|----------|-------------------|
| 1 | Encoding | GBK page không declared charset | High | Detect via byte frequency, fallback to UTF-8 if unknown, mark metadata.language="unknown" |
| 2 | Bot Protection | Turnstile widget trên contact form (non-challenge) | Critical | is_bot_protected() return false (size >100KB); commit 80307d3 fix false positive |
| 3 | Redirect Chain | 6 redirect loop same domain | Medium | Abort after 5, log warning, return partial |
| 4 | SPA Hydration | Next.js page với __NEXT_DATA__ nhưng <30 words visible HTML | High | Data island fallback trigger (CLAUDE.md threshold), extract JSON |
| 5 | Network | 503 retry sau 30s rate limit | Medium | Exponential backoff (1s, 2s, 4s, 8s, 16s), max 5 retry, abort if 429 |

### Summary
- Critical: N
- High: N
- Medium: N
- Low: N
- Total: N scenarios across X dimensions
```

## Kết hợp (CONTEXT)

| Sau scenario | Skill tiếp | Khi nào |
|-------------|-----------|---------|
| Scenarios → test cases | Viết `#[cfg(test)] mod tests` hoặc `tests/*.rs` | Feed Critical/High scenarios vào test target |
| Scenarios → plan risks | wc-cook Step 2 | Paste rows vào risk assessment |
| Critical scenarios | wc-predict | Feed Critical vào 5-persona debate |
| Scenarios cho corpus bench | wc-extraction-bench | Add test corpus case |
