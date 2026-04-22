/// LLM-powered content summarization. Keeps it simple: one function, one prompt.
use crate::clean::strip_thinking_tags;
use crate::error::LlmError;
use crate::provider::{CompletionRequest, LlmProvider, Message};

/// Summarize content using an LLM.
/// Returns plain text (not JSON). Default is 3 sentences.
/// Truncates input to avoid overwhelming small models.
///
/// # Errors
///
/// - `LlmError::ProviderError` — underlying provider HTTP/API failure.
pub async fn summarize(
    content: &str,
    max_sentences: Option<usize>,
    provider: &dyn LlmProvider,
    model: Option<&str>,
) -> Result<String, LlmError> {
    let n = max_sentences.unwrap_or(3);

    // Truncate content to ~4000 chars (~1000 tokens) for small models.
    // Keeps the beginning (title, intro) which is most informative.
    let truncated = if content.len() > 4000 {
        let boundary = content[..4000].rfind(['.', '\n']).unwrap_or(4000);
        &content[..boundary]
    } else {
        content
    };

    let system = format!(
        "You are a strict summarization engine.\n\
         RULES:\n\
         - Output EXACTLY {n} sentences. Not {more} not fewer.\n\
         - Each sentence must be a complete, informative statement.\n\
         - No bullet points, no headings, no code blocks, no markdown.\n\
         - No introductions like \"Here is\" or \"This page\".\n\
         - No questions or suggestions at the end.\n\
         - Plain text only. Start directly with the first sentence.",
        more = n + 1
    );

    // Cap max_tokens: ~40 tokens per sentence is generous. `n` is a
    // user-provided sentence count (small int), saturating cast is safe.
    let token_limit = u32::try_from(n.saturating_mul(40).saturating_add(20)).unwrap_or(u32::MAX);

    let request = CompletionRequest {
        model: model.unwrap_or_default().to_string(),
        messages: vec![
            Message {
                role: "system".into(),
                content: system,
            },
            Message {
                role: "user".into(),
                content: truncated.to_string(),
            },
        ],
        temperature: Some(0.2),
        max_tokens: Some(token_limit),
        json_mode: false,
    };

    let response = provider.complete(&request).await?;

    Ok(strip_thinking_tags(&response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockSummarizer;

    #[async_trait]
    impl LlmProvider for MockSummarizer {
        async fn complete(&self, req: &CompletionRequest) -> Result<String, LlmError> {
            // Verify the prompt is well-formed
            let system = &req.messages[0].content;
            assert!(system.contains("sentences"));
            assert!(system.contains("summarization engine"));
            assert!(!req.json_mode, "summarize should not use json_mode");
            assert!(req.max_tokens.is_some(), "summarize should set max_tokens");
            Ok("This is a test summary.".into())
        }
        async fn is_available(&self) -> bool {
            true
        }
        fn name(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn summarize_returns_text() {
        let result = summarize("Long article content...", None, &MockSummarizer, None)
            .await
            .unwrap();
        assert_eq!(result, "This is a test summary.");
    }

    #[tokio::test]
    async fn summarize_custom_sentence_count() {
        // Verify custom count is passed through
        struct CountChecker;

        #[async_trait]
        impl LlmProvider for CountChecker {
            async fn complete(&self, req: &CompletionRequest) -> Result<String, LlmError> {
                assert!(req.messages[0].content.contains("5 sentences"));
                Ok("Summary.".into())
            }
            async fn is_available(&self) -> bool {
                true
            }
            fn name(&self) -> &str {
                "count_checker"
            }
        }

        summarize("Content", Some(5), &CountChecker, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn summarize_strips_thinking_tags() {
        struct ThinkingMock;

        #[async_trait]
        impl LlmProvider for ThinkingMock {
            async fn complete(&self, _req: &CompletionRequest) -> Result<String, LlmError> {
                Ok("<think>let me analyze this</think>This is the clean summary.".into())
            }
            async fn is_available(&self) -> bool {
                true
            }
            fn name(&self) -> &str {
                "thinking_mock"
            }
        }

        let result = summarize("Some content", None, &ThinkingMock, None)
            .await
            .unwrap();
        assert_eq!(result, "This is the clean summary.");
    }
}
