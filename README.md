# webclaw — ITF Gia Lai Fork

> Fork từ [0xMassi/webclaw](https://github.com/0xMassi/webclaw) v0.3.4 với các bản vá cho **Windows**, **small model (Ollama)**, và **SerpAPI search fallback**.

## Khác biệt với upstream

| Thay đổi | Commit | Mô tả |
|----------|--------|-------|
| **Windows HTTPS fix** | `80307d3` | Thêm `webpki-roots` feature cho wreq — BoringSSL trên Windows không tìm được system cert store |
| **Turnstile false positive** | `80307d3` | Nâng threshold `challenge-platform` detection lên 50KB — tránh nhầm Cloudflare Turnstile widget (contact form) là bot challenge |
| **SerpAPI search** | `9a783c7` | Search tool dùng SerpAPI (Google results) khi không có `WEBCLAW_API_KEY` — 250 query/tháng/free account, scale bằng multi-key |
| **Multi-key rotation** | `330e395` | `SERPAPI_KEY` hỗ trợ comma-separated nhiều keys. Check quota qua Account API trước mỗi search, auto-switch khi hết |
| **Summarize cho small models** | `2c58d7d` | Truncate input 4000 chars, prompt mạnh hơn (cấm markdown/code), max_tokens cap, Ollama pass `num_predict` |
| **Local research pipeline** | `8350746` | search → batch fetch 5-10 URLs → Ollama synthesis (DeepSeek 671B). Fallback: trả raw sources cho Claude Code tự tổng hợp |
| **Enriched search output** | `2d73c36` | Thêm answer box (title+snippet+link), knowledge graph, related questions từ SerpAPI response |

## Cài đặt trên Windows

### Prerequisites

```bash
winget install Rustlang.Rustup
winget install Kitware.CMake
winget install NASM.NASM
winget install LLVM.LLVM
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

### Build

```bash
git clone https://github.com/doivamong/webclaw.git D:\webclaw
cd D:\webclaw

set PATH=%USERPROFILE%\.cargo\bin;C:\Program Files\CMake\bin;C:\Program Files\NASM;C:\Program Files\LLVM\bin;%PATH%
set LIBCLANG_PATH=C:\Program Files\LLVM\bin

cargo build --release --package webclaw-mcp --package webclaw-cli
```

Output:
- `target\release\webclaw-mcp.exe` — MCP server (stdio)
- `target\release\webclaw.exe` — CLI

### Cấu hình MCP cho Claude Code

`.mcp.json` (trong thư mục project):

```json
{
  "mcpServers": {
    "webclaw": {
      "command": "D:\\webclaw\\target\\release\\webclaw-mcp.exe",
      "args": [],
      "env": {
        "OLLAMA_HOST": "http://localhost:11434",
        "SERPAPI_KEY": "key1,key2,key3",
        "OLLAMA_RESEARCH_MODEL": "deepseek-v3.1:671b-cloud"
      }
    }
  }
}
```

### Cấu hình cho Claude Desktop

`%APPDATA%\Claude\claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "webclaw": {
      "command": "D:\\webclaw\\target\\release\\webclaw-mcp.exe",
      "args": [],
      "env": {
        "OLLAMA_HOST": "http://localhost:11434",
        "SERPAPI_KEY": "key1,key2,key3",
        "OLLAMA_RESEARCH_MODEL": "deepseek-v3.1:671b-cloud"
      }
    }
  }
}
```

---

## 10 MCP Tools

| Tool | Mô tả | Yêu cầu |
|------|--------|---------|
| `scrape` | Extract nội dung 1 URL → markdown/llm/json/text | Local, free |
| `crawl` | BFS crawl website (depth + max_pages) | Local, free |
| `map` | Discover URLs từ sitemap/robots.txt | Local, free |
| `batch` | Multi-URL extraction song song (max 100) | Local, free |
| `extract` | Structured extraction qua JSON schema hoặc prompt | Ollama |
| `summarize` | Tóm tắt trang (tuân thủ max_sentences) | Ollama |
| `diff` | So sánh snapshot trước/sau | Local, free |
| `brand` | Extract colors, fonts, logo, favicon | Local, free |
| `search` | Web search (Google results via SerpAPI) | `SERPAPI_KEY` |
| `research` | Deep research: search → fetch → LLM synthesis | `SERPAPI_KEY` + Ollama |

**10/10 tools chạy local** — không cần `WEBCLAW_API_KEY`.

---

## Biến môi trường

| Variable | Mô tả | Bắt buộc? |
|----------|-------|-----------|
| `OLLAMA_HOST` | Ollama URL (default: `http://localhost:11434`) | Không (auto-detect) |
| `OLLAMA_RESEARCH_MODEL` | Model cho research synthesis (default: `deepseek-v3.1:671b-cloud`) | Không |
| `SERPAPI_KEY` | SerpAPI key(s) cho search/research — hỗ trợ comma-separated multi-key | Không (search/research disabled) |
| `WEBCLAW_API_KEY` | Cloud API cho bot bypass (fallback) | Không |
| `WEBCLAW_PROXY` | Single proxy URL | Không |

> **Không dùng OpenAI.** LLM chain: Ollama (chính) → Claude Code tự tổng hợp (khi Ollama down).

---

## Benchmark: WebClaw vs Claude Code built-in

> Đo thực tế trên Windows 11, i5-12400, RTX 4060 8GB, Ollama cloud (DeepSeek V3.1 671B).
> 3 queries: tiếng Việt local, English technical, English niche.
> Mỗi tool test 10 results. Relevance đánh giá thủ công.
> **Benchmark v2** (2026-04-03): sau khi thêm locale-aware params (hl/gl/lr), auto language detection, quota cache.

### Search: `webclaw.search` vs `WebSearch` (3 queries)

| Metric | webclaw.search | WebSearch | Đánh giá |
|--------|:-:|:-:|---|
| **Tốc độ trung bình** | **8.0s** | 14.7s | webclaw nhanh hơn **~45%** (trước: ~15%) |
| **Kết quả/query** | 10 | 10 | Ngang nhau |
| **Relevance trung bình** | **8.7/10** | 8.0/10 | webclaw nhỉnh hơn (trước: 7.7 vs 7.3) |
| **Snippet richness** | **Chi tiết** | Tổng hợp | webclaw: giá, SĐT, tên cụ thể; WebSearch: synthesis paragraph |
| **Claude synthesis** | Claude tự tổng hợp | Claude tự tổng hợp | Cơ chế giống nhau — webclaw raw data phong phú hơn |
| **Answer box/KG** | Có (SerpAPI) | Có (built-in) | Ngang nhau |
| **Niche query (Q3)** | **6/10** relevant | **7/10 + synthesis** | WebSearch vẫn nhỉnh cho niche, nhưng gap thu hẹp |
| **Quota** | 250/tháng/key (scale N keys) | Giới hạn bởi plan token budget | webclaw quota rõ ràng, scale được |
| **Chi phí** | $0 (free accounts) | Tốn token từ plan ($20–200/mo) | WebSearch ẩn chi phí vào token budget |

> **Lưu ý chi phí WebSearch:** WebSearch không miễn phí — mỗi search inject 500–1000+ tokens kết quả vào context, tích lũy qua các turn, tiêu hao chung budget với code/chat. Plan Pro ($20/mo) có ~44K tokens/5h window — heavy search hết quota nhanh hơn đáng kể. webclaw search chi phí $0 thực sự (SerpAPI free accounts), quota minh bạch (biết chính xác còn bao nhiêu), và scale bằng cách thêm free accounts (N keys × 250 queries/tháng).

**Chi tiết 3 queries:**

```
Q1: "cho thuê xe tự lái Gia Lai giá rẻ" (Tiếng Việt, local)
    webclaw:    9.0s  10/10 relevant  ← có giá cụ thể, SĐT, 10/10 đều Gia Lai
    WebSearch: 14.9s   8/10 relevant  ← 1 sai (Bình Định), 1 off-topic (Mioto chung)

Q2: "Flask SQLite connection pooling best practices" (EN, technical)
    webclaw:    7.2s  10/10 relevant  ← SO, Flask docs chính thức, Reddit, tutorials
    WebSearch: 14.6s   9/10 relevant  ← có peewee ORM off-topic

Q3: "wreq BoringSSL webpki-roots Windows TLS fix 2026" (EN, niche)
    webclaw:    7.9s   6/10 relevant  ← tìm được wreq repo + TLS/webpki liên quan
    WebSearch: 14.5s   7/10 relevant  ← tìm được wreq docs + repo + synthesis
```

**So sánh v1 → v2 (sau locale-aware + quota cache):**
```
                    v1 (trước)      v2 (sau)        Cải thiện
Tốc độ TB          15.7s           8.0s            ↓ 49% (quota cache + network)
Relevance Q1       10/10           10/10           giữ nguyên
Relevance Q2        9/10           10/10           ↑ +1 (hl/gl chính xác hơn)
Relevance Q3        4/10            6/10           ↑ +2 (locale params giúp Google)
Relevance TB        7.7/10          8.7/10         ↑ +1.0
```

**Kết luận search:**
- webclaw thắng áp đảo ở **tốc độ (nhanh gấp ~1.8x), tiếng Việt, technical queries, chi phí $0**
- WebSearch nhỉnh nhẹ ở **niche queries** (7/10 vs 6/10), gap chỉ 1 điểm
- Claude Code tự synthesis cho cả hai tool — không có lợi thế synthesis riêng cho bên nào
- Chiến lược: webclaw cho phần lớn use cases (nhanh hơn, chi phí $0, snippet phong phú hơn)

### Research: `webclaw.research` vs Claude Code (WebSearch + 5× WebFetch)

| Metric | webclaw.research | Claude Code native |
|--------|:-:|:-:|
| **Tốc độ** | **~20s** (1 MCP call) | ~60s (6 tool calls) |
| **Sources scraped** | **5 full pages** | 0 (snippets only) hoặc 5 (nếu thêm WebFetch) |
| **Claude tokens** | **~500** (đọc report) | ~7,500 (search + pages + synthesis) |
| **Token savings** | **93%** | baseline |
| **Cấu trúc** | Overview → Findings → Details → Sources | Flat paragraphs |
| **Citations** | [1]-[4] numbered | Inline URLs |
| **Conflict detection** | Có (prompt built-in) | Không |
| **Auto language** | Có (VI query → VI report) | Tùy context |
| **LLM cost** | Ollama free (DeepSeek 671B cloud) | Claude tokens (trả phí) |

### Tổng hợp: 300 queries/tháng (ước tính)

```
                     webclaw              Claude Code native     Tiết kiệm
                     ──────              ──────────────────     ─────────
Search (300×)        300 × 421 = 126K    300 × 538 = 161K      22%
Research (50×)        50 × 500 =  25K     50 × 7,500 = 375K    93%
───────────────────────────────────────────────────────────────────
TỔNG                 151K tokens          536K tokens            72%
```

### Điểm yếu cần lưu ý (trung thực)

| Điểm yếu webclaw | Mức độ | Nguyên nhân | Ghi chú |
|-------------------|--------|-------------|---------|
| Niche query relevance thấp hơn (6/10 vs 7/10) | Nhẹ | Giới hạn cấu trúc: raw Google results vs Claude LLM synthesis | Gap chỉ 1 điểm, webclaw thắng 2/3 queries. Đã thử nghiệm 3 hướng LLM — xem chi tiết bên dưới |
| ~~Không có Claude synthesis~~ | **Không phải điểm yếu** | Claude Code tự synthesis kết quả từ cả webclaw.search lẫn WebSearch — cơ chế giống nhau. webclaw trả raw data phong phú hơn (snippet dài, giá, SĐT) → Claude synthesis chất lượng hơn | Đã loại khỏi danh sách điểm yếu |
| SerpAPI quota 250/tháng/key | Nhẹ | Free tier giới hạn | Multi-account rotation (N keys × 250/tháng), chi phí $0 |
| SerpAPI down → search fail | Thấp | Phụ thuộc external service | Fallback WebSearch tự động |

### Thử nghiệm LLM query improvement (đã reject)

Đã benchmark 3 hướng tiếp cận LLM cho search, tất cả đều **tệ hơn baseline v2**:

```
Approach          Model                    Prompt strategy              TB Relevance  TB Latency
─────────         ─────                    ──────────────               ───────────   ──────────
v2 (baseline)     Không LLM                Không                       8.7/10        8.0s
v3 (rewrite)      deepseek-v3.1:671b       "Rewrite query for Google"  6.7/10 ↓      15.9s ↑
v4 (enrich)       deepseek-v3.1:671b       "Add 1-2 words, keep all"   8.3/10 ↓      14.1s ↑
```

**Phát hiện:** LLM query rewriting/enrichment phản tác dụng cho domain-specific search:
- **Rewrite** (v3): model tổng quát hóa quá mức — bỏ "wreq", thêm "gunicorn" vào SQLite query
- **Enrich** (v4): prompt tối ưu giữ được từ gốc nhưng từ thêm vào ("certificate") kéo Google lệch hướng
- Model nhỏ (qwen3:8b): rewrite sai nghĩa ("xe tự lái"→"xe tải")
- Model lớn (671B): giữ đúng từ nhưng thêm noise, +6s latency mỗi search
- **Kết luận:** Giữ v2 (locale-aware + quota cache) — tối ưu thực tế, không LLM overhead

---

## Rebuild khi update upstream

```bash
cd D:\webclaw
git fetch upstream
git merge upstream/main

# Kill running MCP processes
taskkill /F /IM webclaw-mcp.exe

# Rebuild
set PATH=%USERPROFILE%\.cargo\bin;C:\Program Files\CMake\bin;C:\Program Files\NASM;C:\Program Files\LLVM\bin;%PATH%
set LIBCLANG_PATH=C:\Program Files\LLVM\bin
cargo build --release --package webclaw-mcp --package webclaw-cli
```

---

## Upstream

- **Repo gốc:** [0xMassi/webclaw](https://github.com/0xMassi/webclaw)
- **Docs:** [webclaw.io/docs](https://webclaw.io/docs)
- **Discord:** [discord.gg/KDfd48EpnW](https://discord.gg/KDfd48EpnW)

## License

[AGPL-3.0](LICENSE)
