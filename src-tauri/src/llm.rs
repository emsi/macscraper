use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessage,
        CreateChatCompletionRequest,
    },
    Client,
};
use serde::{Deserialize, Serialize};

/// LLM connection parameters passed from the frontend on each call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCallConfig {
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
}

/// Call the LLM with a fully-resolved prompt (variables already substituted by Svelte).
///
/// Returns the assistant message text from the first choice.
///
/// :param prompt: Fully-resolved prompt string.
/// :param config: LLM connection parameters including endpoint, model and optional API key.
/// :return: The assistant message content string.
#[tauri::command]
pub async fn generate_summary(prompt: String, config: LlmCallConfig) -> Result<String, String> {
    let api_key = config.api_key.unwrap_or_else(|| "no-key".to_string());
    let openai_config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(config.endpoint);
    let client = Client::with_config(openai_config);

    let request = CreateChatCompletionRequest {
        model: config.model,
        messages: vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage::from(prompt),
        )],
        ..Default::default()
    };

    let response = client
        .chat()
        .create(request)
        .await
        .map_err(|e| e.to_string())?;

    response
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .ok_or_else(|| "LLM returned empty response".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_openai_endpoint() {
        let cfg = LlmCallConfig {
            endpoint: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
            api_key: None,
        };
        assert!(cfg.endpoint.contains("openai.com"));
    }

    #[test]
    fn test_llm_config_custom_endpoint() {
        let cfg = LlmCallConfig {
            endpoint: "http://localhost:11434/v1".into(),
            model: "llama3".into(),
            api_key: Some("ollama".into()),
        };
        assert!(cfg.endpoint.contains("11434"));
    }
}
