use super::api_client::{ApiClient, AuthMethod};
use super::errors::ProviderError;
use super::retry::ProviderRetry;
use super::utils::{get_model, handle_response_openai_compat};
use crate::conversation::message::Message;
use crate::impl_provider_default;
use crate::model::ModelConfig;
use crate::providers::base::{ConfigKey, Provider, ProviderMetadata, ProviderUsage, Usage};
use crate::providers::formats::openai::{create_request, get_usage, response_to_message};
use anyhow::Result;
use async_trait::async_trait;
use rmcp::model::Tool;
use serde_json::Value;

pub const SWISS_AI_API_HOST: &str = "https://api.swiss-ai-platform.ch";
pub const SWISS_AI_DEFAULT_MODEL: &str = "llama-3.3-70b-instruct";
pub const SWISS_AI_KNOWN_MODELS: &[&str] = &[
    "llama-3.3-70b-instruct",
    "llama-4-405b-instruct",
];

pub const SWISS_AI_DOC_URL: &str = "https://docs.swiss-ai-platform.ch/models";

#[derive(serde::Serialize)]
pub struct SwissAiProvider {
    #[serde(skip)]
    api_client: ApiClient,
    model: ModelConfig,
}

impl_provider_default!(SwissAiProvider);

impl SwissAiProvider {
    pub fn from_env(model: ModelConfig) -> Result<Self> {
        let config = crate::config::Config::global();
        let api_key: String = config.get_secret("SWISS_AI_API_KEY")?;
        let host: String = config
            .get_param("SWISS_AI_HOST")
            .unwrap_or_else(|_| SWISS_AI_API_HOST.to_string());

        let auth = AuthMethod::BearerToken(api_key);
        let api_client = ApiClient::new(host, auth)?;

        Ok(Self { api_client, model })
    }

    async fn post(&self, payload: Value) -> Result<Value, ProviderError> {
        let response = self
            .api_client
            .response_post("v1/chat/completions", &payload)
            .await?;
        handle_response_openai_compat(response).await
    }
}

#[async_trait]
impl Provider for SwissAiProvider {
    fn metadata() -> ProviderMetadata {
        ProviderMetadata::new(
            "swiss-ai",
            "Swiss AI Platform",
            "Swiss AI Platform with Llama models",
            SWISS_AI_DEFAULT_MODEL,
            SWISS_AI_KNOWN_MODELS.to_vec(),
            SWISS_AI_DOC_URL,
            vec![
                ConfigKey::new("SWISS_AI_API_KEY", true, true, None),
                ConfigKey::new("SWISS_AI_HOST", false, false, Some(SWISS_AI_API_HOST)),
            ],
        )
    }

    fn get_model_config(&self) -> ModelConfig {
        self.model.clone()
    }

    #[tracing::instrument(
        skip(self, system, messages, tools),
        fields(model_config, input, output, input_tokens, output_tokens, total_tokens)
    )]
    async fn complete(
        &self,
        system: &str,
        messages: &[Message],
        tools: &[Tool],
    ) -> Result<(Message, ProviderUsage), ProviderError> {
        let payload = create_request(
            &self.model,
            system,
            messages,
            tools,
            &super::utils::ImageFormat::OpenAi,
        )?;

        let response = self.with_retry(|| self.post(payload.clone())).await?;

        let message = response_to_message(&response)?;
        let usage = response.get("usage").map(get_usage).unwrap_or_else(|| {
            tracing::debug!("Failed to get usage data");
            Usage::default()
        });
        let model = get_model(&response);
        super::utils::emit_debug_trace(&self.model, &payload, &response, &usage);
        Ok((message, ProviderUsage::new(model, usage)))
    }

    /// Fetch supported models from Swiss AI Platform; returns Err on failure, Ok(None) if no models found
    async fn fetch_supported_models(&self) -> Result<Option<Vec<String>>, ProviderError> {
        let response = self
            .api_client
            .request("v1/models")
            .header("Content-Type", "application/json")?
            .response_get()
            .await?;
        let response = handle_response_openai_compat(response).await?;

        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ProviderError::UsageError("Missing or invalid `data` field in response".into())
            })?;

        let mut model_names: Vec<String> = data
            .iter()
            .filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(String::from))
            .collect();
        model_names.sort();
        Ok(Some(model_names))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModelConfig;

    #[test]
    fn test_provider_metadata() {
        let metadata = SwissAiProvider::metadata();
        assert_eq!(metadata.name, "swiss-ai");
        assert_eq!(metadata.display_name, "Swiss AI Platform");
        assert_eq!(metadata.description, "Swiss AI Platform with Llama models");
        assert_eq!(metadata.default_model, SWISS_AI_DEFAULT_MODEL);
        assert_eq!(metadata.model_doc_link, SWISS_AI_DOC_URL);
    }

    #[test]
    fn test_known_models() {
        let metadata = SwissAiProvider::metadata();
        assert!(metadata.known_models.iter()
            .any(|m| m.name == "llama-3.3-70b-instruct"));
        assert!(metadata.known_models.iter()
            .any(|m| m.name == "llama-4-405b-instruct"));
        assert_eq!(metadata.known_models.len(), 2);
    }

    #[test]
    fn test_config_keys() {
        let metadata = SwissAiProvider::metadata();
        assert_eq!(metadata.config_keys.len(), 2);
        
        let api_key_config = metadata.config_keys.iter()
            .find(|k| k.name == "SWISS_AI_API_KEY")
            .expect("SWISS_AI_API_KEY config should exist");
        assert!(api_key_config.required);
        assert!(api_key_config.secret);
        
        let host_config = metadata.config_keys.iter()
            .find(|k| k.name == "SWISS_AI_HOST")
            .expect("SWISS_AI_HOST config should exist");
        assert!(!host_config.required);
        assert!(!host_config.secret);
        assert_eq!(host_config.default, Some(SWISS_AI_API_HOST.to_string()));
    }

    #[test]
    fn test_from_env_missing_api_key() {
        std::env::remove_var("SWISS_AI_API_KEY");
        let model = ModelConfig::new_or_fail("llama-3.3-70b-instruct");
        let result = SwissAiProvider::from_env(model);
        assert!(result.is_err());
    }

    #[test]
    fn test_constants() {
        assert_eq!(SWISS_AI_API_HOST, "https://api.swiss-ai-platform.ch");
        assert_eq!(SWISS_AI_DEFAULT_MODEL, "llama-3.3-70b-instruct");
        assert_eq!(SWISS_AI_DOC_URL, "https://docs.swiss-ai-platform.ch/models");
        assert_eq!(SWISS_AI_KNOWN_MODELS.len(), 2);
        assert!(SWISS_AI_KNOWN_MODELS.contains(&"llama-3.3-70b-instruct"));
        assert!(SWISS_AI_KNOWN_MODELS.contains(&"llama-4-405b-instruct"));
    }
}