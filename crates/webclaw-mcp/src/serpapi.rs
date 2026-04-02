/// SerpAPI local search with multi-key rotation.
///
/// Supports comma-separated keys in SERPAPI_KEY env var.
/// Checks quota via Account API (free, not counted) before each search.
/// Auto-rotates to next key when current key is exhausted.
use serde_json::Value;
use tracing::{info, warn};

const SEARCH_URL: &str = "https://serpapi.com/search.json";
const ACCOUNT_URL: &str = "https://serpapi.com/account.json";

/// SerpAPI client with multi-key rotation.
pub struct SerpApiClient {
    keys: Vec<String>,
    http: reqwest::Client,
}

impl SerpApiClient {
    /// Create from SERPAPI_KEY env var (comma-separated for multiple keys).
    /// Returns None if not set or empty.
    pub fn from_env() -> Option<Self> {
        let raw = std::env::var("SERPAPI_KEY").ok()?;
        let keys: Vec<String> = raw
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if keys.is_empty() {
            return None;
        }
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        info!(key_count = keys.len(), "SerpAPI search enabled (multi-key rotation)");
        Some(Self { keys, http })
    }

    /// Check remaining quota for a key. Returns searches left, or 0 on error.
    async fn check_quota(&self, key: &str) -> u64 {
        let resp = self
            .http
            .get(ACCOUNT_URL)
            .query(&[("api_key", key)])
            .send()
            .await;
        match resp {
            Ok(r) if r.status().is_success() => {
                if let Ok(data) = r.json::<Value>().await {
                    data.get("total_searches_left")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    /// Find the first key with remaining quota.
    async fn pick_key(&self) -> Option<(usize, &str)> {
        for (i, key) in self.keys.iter().enumerate() {
            let left = self.check_quota(key).await;
            if left > 0 {
                info!(key_index = i, searches_left = left, "using SerpAPI key");
                return Some((i, key));
            }
            warn!(key_index = i, "SerpAPI key exhausted, trying next");
        }
        None
    }

    /// Search Google via SerpAPI with auto key rotation.
    pub async fn search(&self, query: &str, num_results: Option<u32>) -> Result<String, String> {
        let (idx, key) = self
            .pick_key()
            .await
            .ok_or_else(|| {
                format!(
                    "All {} SerpAPI keys exhausted. Add more keys to SERPAPI_KEY (comma-separated).",
                    self.keys.len()
                )
            })?;

        let num = num_results.unwrap_or(10).min(20);

        let resp = self
            .http
            .get(SEARCH_URL)
            .query(&[
                ("q", query),
                ("api_key", key),
                ("engine", "google"),
                ("num", &num.to_string()),
            ])
            .send()
            .await
            .map_err(|e| format!("SerpAPI request failed: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!(
                "SerpAPI error {status} (key #{idx}): {}",
                &text[..text.len().min(300)]
            ));
        }

        let data: Value = resp
            .json()
            .await
            .map_err(|e| format!("SerpAPI parse failed: {e}"))?;

        format_results(&data)
    }
}

/// Format SerpAPI response into readable text.
fn format_results(data: &Value) -> Result<String, String> {
    let mut output = String::new();

    if let Some(results) = data.get("organic_results").and_then(|v| v.as_array()) {
        output.push_str(&format!("Found {} results:\n\n", results.len()));
        for (i, result) in results.iter().enumerate() {
            let title = result.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let url = result.get("link").and_then(|v| v.as_str()).unwrap_or("");
            let snippet = result.get("snippet").and_then(|v| v.as_str()).unwrap_or("");
            output.push_str(&format!("{}. {}\n   {}\n   {}\n\n", i + 1, title, url, snippet));
        }
    } else {
        output.push_str("No results found.\n");
    }

    // Append answer box if present
    if let Some(answer) = data.get("answer_box") {
        if let Some(snippet) = answer.get("snippet").and_then(|v| v.as_str()) {
            output.push_str(&format!("--- Answer Box ---\n{}\n\n", snippet));
        }
    }

    Ok(output)
}
