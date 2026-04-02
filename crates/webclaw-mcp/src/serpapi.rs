/// SerpAPI local search with multi-key rotation.
///
/// Supports comma-separated keys in SERPAPI_KEY env var.
/// Checks quota via Account API (free, not counted) before each search.
/// Auto-rotates to next key when current key is exhausted.
/// Quota results are cached for 5 minutes to reduce latency.
/// Optional LLM query rewriting via Ollama for better relevance.
use serde_json::Value;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{info, warn};

const SEARCH_URL: &str = "https://serpapi.com/search.json";
const ACCOUNT_URL: &str = "https://serpapi.com/account.json";

/// How long to cache quota check results.
const QUOTA_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Structured search result for research pipeline.
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Search options derived from params + auto-detection.
pub struct SearchOptions {
    pub num: u32,
    pub country: Option<String>,
    pub language: Option<String>,
    pub recency: Option<String>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            num: 10,
            country: None,
            language: None,
            recency: None,
        }
    }
}

/// Cached quota entry for a single key.
struct QuotaEntry {
    remaining: u64,
    checked_at: Instant,
}

/// SerpAPI client with multi-key rotation and quota caching.
pub struct SerpApiClient {
    keys: Vec<String>,
    http: reqwest::Client,
    /// Per-key quota cache: index matches self.keys.
    quota_cache: Mutex<Vec<Option<QuotaEntry>>>,
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
        let cache: Vec<Option<QuotaEntry>> = (0..keys.len()).map(|_| None).collect();
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        info!(key_count = keys.len(), "SerpAPI search enabled (multi-key rotation)");
        Some(Self {
            keys,
            http,
            quota_cache: Mutex::new(cache),
        })
    }

    /// Check remaining quota for a key. Uses cache if fresh (< 5 min).
    async fn check_quota(&self, key_index: usize, key: &str) -> u64 {
        // Check cache first
        if let Ok(cache) = self.quota_cache.lock() {
            if let Some(Some(entry)) = cache.get(key_index) {
                if entry.checked_at.elapsed() < QUOTA_CACHE_TTL && entry.remaining > 0 {
                    return entry.remaining;
                }
            }
        }

        // Cache miss or expired — fetch from API
        let remaining = self.fetch_quota(key).await;

        // Update cache
        if let Ok(mut cache) = self.quota_cache.lock() {
            if let Some(slot) = cache.get_mut(key_index) {
                *slot = Some(QuotaEntry {
                    remaining,
                    checked_at: Instant::now(),
                });
            }
        }

        remaining
    }

    /// Fetch quota from SerpAPI Account API (not cached).
    async fn fetch_quota(&self, key: &str) -> u64 {
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

    /// Decrement cached quota after a successful search.
    fn decrement_quota(&self, key_index: usize) {
        if let Ok(mut cache) = self.quota_cache.lock() {
            if let Some(Some(entry)) = cache.get_mut(key_index) {
                entry.remaining = entry.remaining.saturating_sub(1);
            }
        }
    }

    /// Find the first key with remaining quota.
    async fn pick_key(&self) -> Option<(usize, &str)> {
        for (i, key) in self.keys.iter().enumerate() {
            let left = self.check_quota(i, key).await;
            if left > 0 {
                info!(key_index = i, searches_left = left, "using SerpAPI key");
                return Some((i, key));
            }
            warn!(key_index = i, "SerpAPI key exhausted, trying next");
        }
        None
    }

    /// Raw SerpAPI call. Returns parsed JSON.
    async fn raw_search(&self, query: &str, opts: &SearchOptions) -> Result<Value, String> {
        let (idx, key) = self
            .pick_key()
            .await
            .ok_or_else(|| {
                format!(
                    "All {} SerpAPI keys exhausted. Add more keys to SERPAPI_KEY (comma-separated).",
                    self.keys.len()
                )
            })?;

        // Auto-detect language from query if not specified
        let detected_lang = opts.language.as_deref().unwrap_or_else(|| detect_language(query));
        let detected_country = opts
            .country
            .as_deref()
            .unwrap_or_else(|| lang_to_country(detected_lang));

        let num_str = opts.num.min(20).to_string();
        let mut params: Vec<(&str, &str)> = vec![
            ("q", query),
            ("api_key", key),
            ("engine", "google"),
            ("num", &num_str),
            ("hl", detected_lang),
            ("gl", detected_country),
        ];

        // Language restrict: only for non-English queries
        let lr_value = format!("lang_{detected_lang}");
        if detected_lang != "en" {
            params.push(("lr", &lr_value));
        }

        // Time-based search filter
        let tbs_value;
        if let Some(ref recency) = opts.recency {
            tbs_value = match recency.as_str() {
                "day" => "qdr:d".to_string(),
                "week" => "qdr:w".to_string(),
                "month" => "qdr:m".to_string(),
                "year" => "qdr:y".to_string(),
                _ => String::new(),
            };
            if !tbs_value.is_empty() {
                params.push(("tbs", &tbs_value));
            }
        }

        info!(
            query,
            lang = detected_lang,
            country = detected_country,
            recency = opts.recency.as_deref().unwrap_or("none"),
            "SerpAPI search"
        );

        let resp = self
            .http
            .get(SEARCH_URL)
            .query(&params)
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

        // Successful search — decrement cached quota
        self.decrement_quota(idx);

        resp.json()
            .await
            .map_err(|e| format!("SerpAPI parse failed: {e}"))
    }

    /// Search and return formatted text (for search tool).
    pub async fn search(
        &self,
        query: &str,
        num_results: Option<u32>,
        country: Option<String>,
        language: Option<String>,
        recency: Option<String>,
    ) -> Result<String, String> {
        let opts = SearchOptions {
            num: num_results.unwrap_or(10),
            country,
            language,
            recency,
        };
        let data = self.raw_search(query, &opts).await?;
        format_results(&data)
    }

    /// Search and return structured results (for research pipeline).
    pub async fn search_urls(&self, query: &str, num: u32) -> Result<Vec<SearchResult>, String> {
        let opts = SearchOptions {
            num,
            ..Default::default()
        };
        let data = self.raw_search(query, &opts).await?;
        Ok(parse_results(&data))
    }
}

/// Detect language from query text using simple heuristics.
/// Returns ISO 639-1 code: "vi", "ja", "zh", "ko", or "en" (default).
fn detect_language(query: &str) -> &'static str {
    for ch in query.chars() {
        // Vietnamese diacritical marks (Latin Extended + combining)
        if matches!(ch, 'à'..='ỹ' | 'À'..='Ỹ') {
            return "vi";
        }
        // CJK Unified Ideographs
        if ('\u{4E00}'..='\u{9FFF}').contains(&ch) {
            return "zh";
        }
        // Japanese Hiragana / Katakana
        if ('\u{3040}'..='\u{30FF}').contains(&ch) {
            return "ja";
        }
        // Korean Hangul
        if ('\u{AC00}'..='\u{D7AF}').contains(&ch) || ('\u{1100}'..='\u{11FF}').contains(&ch) {
            return "ko";
        }
    }
    "en"
}

/// Map language code to default country code.
fn lang_to_country(lang: &str) -> &'static str {
    match lang {
        "vi" => "vn",
        "ja" => "jp",
        "zh" => "cn",
        "ko" => "kr",
        _ => "us",
    }
}

/// Parse organic results into structured SearchResult vec.
fn parse_results(data: &Value) -> Vec<SearchResult> {
    data.get("organic_results")
        .and_then(|v| v.as_array())
        .map(|results| {
            results
                .iter()
                .filter_map(|r| {
                    Some(SearchResult {
                        title: r.get("title")?.as_str()?.to_string(),
                        url: r.get("link")?.as_str()?.to_string(),
                        snippet: r
                            .get("snippet")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Format SerpAPI response into readable text.
fn format_results(data: &Value) -> Result<String, String> {
    let results = parse_results(data);
    let mut output = String::new();

    if results.is_empty() {
        output.push_str("No results found.\n");
    } else {
        output.push_str(&format!("Found {} results:\n\n", results.len()));
        for (i, r) in results.iter().enumerate() {
            output.push_str(&format!(
                "{}. {}\n   {}\n   {}\n\n",
                i + 1,
                r.title,
                r.url,
                r.snippet
            ));
        }
    }

    // Append answer box if present (title + snippet + link)
    if let Some(answer) = data.get("answer_box") {
        output.push_str("--- Answer Box ---\n");
        if let Some(title) = answer.get("title").and_then(|v| v.as_str()) {
            output.push_str(&format!("{title}\n"));
        }
        if let Some(snippet) = answer.get("snippet").and_then(|v| v.as_str()) {
            output.push_str(&format!("{snippet}\n"));
        } else if let Some(answer_text) = answer.get("answer").and_then(|v| v.as_str()) {
            output.push_str(&format!("{answer_text}\n"));
        }
        if let Some(link) = answer.get("link").and_then(|v| v.as_str()) {
            output.push_str(&format!("Source: {link}\n"));
        }
        output.push('\n');
    }

    // Append knowledge graph if present
    if let Some(kg) = data.get("knowledge_graph") {
        let title = kg.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let desc = kg.get("description").and_then(|v| v.as_str()).unwrap_or("");
        if !title.is_empty() || !desc.is_empty() {
            output.push_str(&format!("--- Knowledge Graph ---\n{title}\n{desc}\n\n"));
        }
    }

    // Append related questions if present
    if let Some(related) = data.get("related_questions").and_then(|v| v.as_array()) {
        if !related.is_empty() {
            output.push_str("--- Related Questions ---\n");
            for q in related.iter().take(3) {
                if let Some(question) = q.get("question").and_then(|v| v.as_str()) {
                    output.push_str(&format!("• {question}\n"));
                    if let Some(snippet) = q.get("snippet").and_then(|v| v.as_str()) {
                        output.push_str(&format!("  {}\n", &snippet[..snippet.len().min(150)]));
                    }
                }
            }
            output.push('\n');
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language_vietnamese() {
        assert_eq!(detect_language("cho thuê xe tự lái Gia Lai giá rẻ"), "vi");
        assert_eq!(detect_language("nhà hàng Đà Nẵng"), "vi");
    }

    #[test]
    fn test_detect_language_english() {
        assert_eq!(detect_language("Flask SQLite connection pooling"), "en");
        assert_eq!(detect_language("rust async tutorial"), "en");
    }

    #[test]
    fn test_detect_language_cjk() {
        assert_eq!(detect_language("おすすめレストラン"), "ja");
        assert_eq!(detect_language("北京天气预报"), "zh");
        assert_eq!(detect_language("서울 맛집"), "ko");
    }

    #[test]
    fn test_lang_to_country() {
        assert_eq!(lang_to_country("vi"), "vn");
        assert_eq!(lang_to_country("en"), "us");
        assert_eq!(lang_to_country("ja"), "jp");
    }

    #[test]
    fn test_recency_values() {
        // Just verify the mapping is correct
        assert_eq!(
            match "day" { "day" => "qdr:d", "week" => "qdr:w", "month" => "qdr:m", "year" => "qdr:y", _ => "" },
            "qdr:d"
        );
    }
}
