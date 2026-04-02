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
> Cùng query: `"cho thuê xe tự lái Gia Lai giá rẻ"` (search) và `"Flask SQLite WAL mode best practices"` (research).

### Search: `webclaw.search` vs `WebSearch`

```
                     webclaw.search              WebSearch (built-in)
                     ──────────────              ────────────────────
Tốc độ               ~19s                        ~17s
Kết quả              5 (Google, SerpAPI)         10 (Google, built-in)
Token vào context    ~230                        ~675 (results + synthesis)
Claude synthesis     0 (kết quả sẵn dùng)       ~375 tokens (Claude tổng hợp)
Tổng Claude tokens   ~230                        ~675
Tiết kiệm token      ██████████████████ 66%      ── baseline ──
Chi phí              1 SerpAPI query / 500/tháng  Included in plan
```

### Research: `webclaw.research` vs Claude Code (WebSearch + 5× WebFetch)

```
                     webclaw.research             Claude Code native
                     ────────────────             ──────────────────
Tốc độ               ~20s (1 call)               ~60s (6 calls)
Sources scraped       5 (full content)            0 hoặc 5 (nếu thêm WebFetch)
Token vào context    ~500 (report)               ~7,500 (search + 5 pages + synthesis)
Claude synthesis     0 (Ollama, free)            ~500 tokens
Tổng Claude tokens   ~500                        ~7,500
Tiết kiệm token      █████████████████████ 93%   ── baseline ──
Cấu trúc report     Overview → Findings →        Flat paragraphs
                     Details → Sources [N]
Citations            [1]-[4] numbered             Inline URLs
Conflict detection   Có (prompt built-in)         Không
Auto-detect language Có (VI query → VI report)    Tùy context
```

### Tổng hợp: 300 queries/tháng

```
                     webclaw              Claude Code native
                     ──────              ──────────────────
Search tokens        300 × 230 = 69K     300 × 675 = 203K
Research tokens      50 × 500 = 25K      50 × 7,500 = 375K
──────────────────────────────────────────────────────────
TỔNG                 94K tokens/tháng     578K tokens/tháng
Tiết kiệm            ████████████████████ 84%
```

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
