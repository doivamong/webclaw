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
        "SERPAPI_KEY": "key1,key2,key3"
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
        "SERPAPI_KEY": "key1,key2,key3"
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
