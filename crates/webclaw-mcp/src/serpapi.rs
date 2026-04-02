/// SerpAPI local search fallback.
///
/// When WEBCLAW_API_KEY is not set but SERPAPI_KEY is available,
/// use SerpAPI (serpapi.com) for web search with Google results.
/// 250 queries/month free tier.
use serde_json::Value;
use tracing::info;

const API_BASE: &str = "https://serpapi.com/search.json";

/// Lightweight SerpAPI client.
pub struct SerpApiClient {
    api_key: String,
    http: reqwest::Client,
}

impl SerpApiClient {
    /// Create from SERPAPI_KEY env var. Returns None if not set.
    pub fn from_env() -> Option<Self> {
        let key = std::env::var("SERPAPI_KEY").ok()?;
        if key.is_empty() {
            return None;
        }
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        info!("SerpAPI search enabled (SERPAPI_KEY set)");
        Some(Self { api_key: key, http })
    }

    /// Search Google via SerpAPI. Returns formatted results string.
    pub async fn search(&self, query: &str, num_results: Option<u32>) -> Result<String, String> {
        let num = num_results.unwrap_or(10).min(20);

        let resp = self
            .http
            .get(API_BASE)
            .query(&[
                ("q", query),
                ("api_key", &self.api_key),
                ("engine", "google"),
                ("num", &num.to_string()),
            ])
            .send()
            .await
            .map_err(|e| format!("SerpAPI request failed: {e}"))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("SerpAPI error {status}: {}", &text[..text.len().min(300)]));
        }

        let data: Value = resp
            .json()
            .await
            .map_err(|e| format!("SerpAPI parse failed: {e}"))?;

        // Format organic results
        let mut output = String::new();
        if let Some(results) = data.get("organic_results").and_then(|v| v.as_array()) {
            output.push_str(&format!("Found {} results:\n\n", results.len()));
            for (i, result) in results.iter().enumerate() {
                let title = result.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let url = result.get("link").and_then(|v| v.as_str()).unwrap_or("");
                let snippet = result.get("snippet").and_then(|v| v.as_str()).unwrap_or("");

                output.push_str(&format!(
                    "{}. {}\n   {}\n   {}\n\n",
                    i + 1,
                    title,
                    url,
                    snippet
                ));
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
}
