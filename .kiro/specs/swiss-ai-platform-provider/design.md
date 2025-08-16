# Design Document

## Overview

Der Swiss AI Platform Provider wird als neuer Provider in das Goose-Framework integriert, um Zugang zu Llama-3.3 und Llama-4 Modellen über eine OpenAI-kompatible API zu ermöglichen. Das Design folgt dem etablierten Provider-Pattern und nutzt die bestehende OpenAI-kompatible Infrastruktur.

## Architecture

### High-Level Architecture

```
┌─────────────────┐    ┌──────────────────────┐    ┌─────────────────────┐
│   Goose Core    │───▶│ SwissAiProvider      │───▶│ Swiss AI Platform   │
│                 │    │                      │    │ API                 │
│ - Factory       │    │ - Authentication     │    │                     │
│ - Provider      │    │ - Request Formatting │    │ - Llama-3.3         │
│   Interface     │    │ - Response Parsing   │    │ - Llama-4           │
│                 │    │ - Error Handling     │    │                     │
└─────────────────┘    └──────────────────────┘    └─────────────────────┘
```

### Provider Integration

Der SwissAiProvider wird in die bestehende Provider-Factory integriert:

1. **Provider Registration**: Registrierung in `providers/factory.rs`
2. **Metadata Definition**: Konfiguration der Provider-Metadaten
3. **Module Export**: Export über `providers/mod.rs`

## Components and Interfaces

### 1. SwissAiProvider Struct

```rust
#[derive(serde::Serialize)]
pub struct SwissAiProvider {
    #[serde(skip)]
    api_client: ApiClient,
    model: ModelConfig,
}
```

**Responsibilities:**
- API-Client-Management
- Modell-Konfiguration
- Request/Response-Handling

### 2. Provider Trait Implementation

```rust
#[async_trait]
impl Provider for SwissAiProvider {
    fn metadata() -> ProviderMetadata;
    fn get_model_config(&self) -> ModelConfig;
    async fn complete(&self, system: &str, messages: &[Message], tools: &[Tool]) 
        -> Result<(Message, ProviderUsage), ProviderError>;
    async fn fetch_supported_models(&self) -> Result<Option<Vec<String>>, ProviderError>;
}
```

### 3. Configuration Constants

```rust
pub const SWISS_AI_API_HOST: &str = "https://api.swiss-ai-platform.ch";
pub const SWISS_AI_DEFAULT_MODEL: &str = "llama-3.3-70b-instruct";
pub const SWISS_AI_KNOWN_MODELS: &[&str] = &[
    "llama-3.3-70b-instruct",
    "llama-4-405b-instruct",
];
pub const SWISS_AI_DOC_URL: &str = "https://docs.swiss-ai-platform.ch/models";
```

### 4. Factory Integration

```rust
// In providers/factory.rs
use super::swiss_ai::SwissAiProvider;

pub fn providers() -> Vec<ProviderMetadata> {
    vec![
        // ... existing providers
        SwissAiProvider::metadata(),
    ]
}

pub fn create_provider(name: &str, model: ModelConfig) -> Result<Arc<dyn Provider>> {
    match name {
        // ... existing providers
        "swiss-ai" => Ok(Arc::new(SwissAiProvider::from_env(model)?)),
        _ => Err(anyhow!("Unknown provider: {}", name)),
    }
}
```

## Data Models

### 1. Provider Metadata

```rust
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
```

### 2. Model Information

```rust
// Modell-Definitionen mit Context-Limits und Kosten
let known_models = vec![
    ModelInfo::new("llama-3.3-70b-instruct", 128000),
    ModelInfo::new("llama-4-405b-instruct", 128000),
];
```

### 3. API Request/Response Format

Da die Swiss AI Platform OpenAI-kompatibel ist, werden die bestehenden OpenAI-Format-Utilities verwendet:

```rust
// Request creation
let payload = create_request(
    &self.model,
    system,
    messages,
    tools,
    &ImageFormat::OpenAi,
)?;

// Response parsing
let message = response_to_message(&response)?;
let usage = response.get("usage").map(get_usage).unwrap_or_default();
```

## Error Handling

### 1. Error Types

Der Provider nutzt die bestehenden `ProviderError`-Typen:

- **AuthenticationError**: Ungültige API-Keys
- **NetworkError**: Verbindungsprobleme
- **RateLimitError**: API-Rate-Limits
- **UsageError**: Ungültige Requests oder Responses

### 2. Error Mapping

```rust
async fn post(&self, payload: Value) -> Result<Value, ProviderError> {
    let response = self
        .api_client
        .response_post("v1/chat/completions", &payload)
        .await?;
    handle_response_openai_compat(response).await
}
```

### 3. Retry Strategy

Der Provider nutzt das bestehende Retry-System:

```rust
let response = self.with_retry(|| self.post(payload.clone())).await?;
```

## Testing Strategy

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_metadata() {
        let metadata = SwissAiProvider::metadata();
        assert_eq!(metadata.name, "swiss-ai");
        assert_eq!(metadata.display_name, "Swiss AI Platform");
    }
    
    #[test]
    fn test_known_models() {
        let metadata = SwissAiProvider::metadata();
        assert!(metadata.known_models.iter()
            .any(|m| m.name == "llama-3.3-70b-instruct"));
    }
}
```

### 2. Integration Tests

```rust
#[tokio::test]
async fn test_complete_request() {
    // Test mit Mock-API-Server
    let provider = SwissAiProvider::from_env(test_model_config())?;
    let result = provider.complete("Test system", &[], &[]).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_fetch_supported_models() {
    let provider = SwissAiProvider::from_env(test_model_config())?;
    let models = provider.fetch_supported_models().await?;
    assert!(models.is_some());
}
```

### 3. Configuration Tests

```rust
#[test]
fn test_from_env_with_valid_config() {
    std::env::set_var("SWISS_AI_API_KEY", "test-key");
    let result = SwissAiProvider::from_env(test_model_config());
    assert!(result.is_ok());
}

#[test]
fn test_from_env_missing_api_key() {
    std::env::remove_var("SWISS_AI_API_KEY");
    let result = SwissAiProvider::from_env(test_model_config());
    assert!(result.is_err());
}
```

## Implementation Details

### 1. File Structure

```
crates/goose/src/providers/
├── swiss_ai.rs          # Haupt-Provider-Implementation
├── mod.rs               # Module-Export hinzufügen
└── factory.rs           # Provider-Registration
```

### 2. Dependencies

Keine neuen Dependencies erforderlich - nutzt bestehende:
- `ApiClient` für HTTP-Requests
- `openai` Format-Utilities für Request/Response-Handling
- `base` Provider-Traits und Strukturen

### 3. Configuration

```rust
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
}
```

### 4. API Endpoints

- **Chat Completions**: `POST /v1/chat/completions`
- **Models List**: `GET /v1/models`

### 5. Authentication

Bearer Token Authentication über `Authorization: Bearer <api_key>` Header.

## Security Considerations

1. **API Key Management**: Sichere Speicherung über Umgebungsvariablen
2. **Request Validation**: Validierung aller eingehenden Parameter
3. **Response Sanitization**: Sichere Verarbeitung der API-Responses
4. **Rate Limiting**: Respektierung der API-Rate-Limits
5. **Error Information**: Keine sensitiven Daten in Fehlermeldungen

## Performance Considerations

1. **Connection Reuse**: HTTP-Client mit Connection-Pooling
2. **Retry Logic**: Exponential Backoff für temporäre Fehler
3. **Timeout Handling**: Angemessene Timeouts für API-Calls
4. **Memory Usage**: Effiziente Serialisierung/Deserialisierung

## Monitoring and Observability

1. **Tracing**: Integration mit dem bestehenden Tracing-System
2. **Metrics**: Usage-Tracking für Kosten und Performance
3. **Error Logging**: Strukturierte Fehlerprotokollierung
4. **Debug Information**: Detaillierte Debug-Ausgaben für Entwicklung