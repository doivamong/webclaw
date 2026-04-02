# webclaw — ITF Gia Lai Fork

> Fork từ [0xMassi/webclaw](https://github.com/0xMassi/webclaw) v0.3.4 với các bản vá cho **Windows**, **small model (Ollama)**, và **SerpAPI search fallback**.

## Khác biệt với upstream

| Thay đổi | Commit | Mô tả |
|----------|--------|-------|
| **Windows HTTPS fix** | `80307d3` | Thêm `webpki-roots` feature cho wreq — BoringSSL trên Windows không tìm được system cert store |
| **Turnstile false positive** | `80307d3` | Nâng threshold `challenge-platform` detection lên 50KB — tránh nhầm Cloudflare Turnstile widget (contact form) là bot challenge |
| **SerpAPI search** | `9a783c7` | Search tool dùng SerpAPI (Google results) khi không có `WEBCLAW_API_KEY` — 250 query/tháng free |
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

### Search: `webclaw.search` vs `WebSearch` (3 queries)

| Metric | webclaw.search | WebSearch | Đánh giá |
|--------|:-:|:-:|---|
| **Tốc độ trung bình** | **15.7s** | 18.0s | webclaw nhanh hơn ~15% |
| **Kết quả/query** | 10 | 10 | Ngang nhau |
| **Relevance trung bình** | **7.7/10** | 7.3/10 | webclaw nhỉnh hơn |
| **Snippet richness** | **1,686 chars** | 1,077 chars | webclaw snippet dài, có giá/SĐT |
| **Claude tokens** | **~421** | ~538 | webclaw tiết kiệm ~22% |
| **Claude synthesis cần?** | Không | Có | webclaw trả kết quả dùng ngay |
| **Answer box/KG** | Có (SerpAPI) | Có (built-in) | Ngang nhau |
| **Niche query (Q3)** | 4/10 relevant | **5/10 + synthesis** | WebSearch tốt hơn cho niche |
| **Quota** | 500/tháng (2 keys) | Không giới hạn | WebSearch linh hoạt hơn |
| **Chi phí** | 1 SerpAPI query | Included in plan | WebSearch miễn phí |

**Chi tiết 3 queries:**

```
Q1: "cho thuê xe tự lái Gia Lai giá rẻ" (Tiếng Việt, local)
    webclaw:   20.6s  10/10 relevant  1,847 chars  ← có giá, SĐT thực
    WebSearch: 17.8s   8/10 relevant  1,050 chars  ← 2 kết quả off-topic

Q2: "Flask SQLite connection pooling best practices" (EN, technical)
    webclaw:   15.7s   9/10 relevant  1,690 chars  ← SO, Flask docs, Reddit
    WebSearch: 17.9s   9/10 relevant  1,200 chars  ← Flask docs, SQLAlchemy

Q3: "wreq BoringSSL webpki-roots Windows TLS fix 2026" (EN, niche)
    webclaw:   10.9s   4/10 relevant  1,520 chars  ← raw results, ít liên quan
    WebSearch: 18.4s   5/10 relevant    980 chars  ← Claude tổng hợp câu trả lời
```

**Kết luận search:**
- webclaw thắng ở **tốc độ, snippet quality, token efficiency, tiếng Việt**
- WebSearch thắng ở **niche queries (Claude synthesis), không giới hạn quota**
- Chiến lược tối ưu: webclaw cho research/kỹ thuật, WebSearch cho niche/unlimited

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

| Điểm yếu webclaw | Mức độ | Workaround |
|-------------------|--------|------------|
| Niche query relevance thấp hơn | Nhẹ | Dùng WebSearch cho niche queries |
| SerpAPI quota 500/tháng | Trung bình | Thêm free accounts, hoặc fallback WebSearch |
| Không có Claude synthesis | Nhẹ | Claude Code tự tổng hợp từ raw results |
| SerpAPI down → search fail | Thấp | Fallback WebSearch tự động |

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
