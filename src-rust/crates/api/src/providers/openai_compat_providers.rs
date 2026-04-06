// providers/openai_compat_providers.rs — Factory functions for all
// OpenAI-compatible provider instances.
//
// Each function constructs a pre-configured [`OpenAiCompatProvider`] for a
// specific service.  API keys are read from environment variables; if the
// variable is absent or empty the provider is still constructed but
// `health_check()` will return `ProviderStatus::Unavailable`.

use claurst_core::provider_id::ProviderId;

use super::openai_compat::{OpenAiCompatProvider, ProviderQuirks};

// ---------------------------------------------------------------------------
// Local / self-hosted providers (no API key required)
// ---------------------------------------------------------------------------

/// Ollama — local inference server.
/// Reads `OLLAMA_HOST` for the base URL; defaults to `http://localhost:11434`.
pub fn ollama() -> OpenAiCompatProvider {
    let host = std::env::var("OLLAMA_HOST")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    let base_url = format!("{}/v1", host.trim_end_matches('/'));
    OpenAiCompatProvider::new(ProviderId::OLLAMA, "Ollama", base_url).with_quirks(
        ProviderQuirks {
            overflow_patterns: vec![
                "prompt too long".to_string(),
                "exceeded.*context length".to_string(),
            ],
            ..Default::default()
        },
    )
}

/// LM Studio — local OpenAI-compatible server.
/// Reads `LM_STUDIO_HOST` for the base URL; defaults to `http://localhost:1234`.
pub fn lm_studio() -> OpenAiCompatProvider {
    let host = std::env::var("LM_STUDIO_HOST")
        .unwrap_or_else(|_| "http://localhost:1234".to_string());
    let base_url = format!("{}/v1", host.trim_end_matches('/'));
    OpenAiCompatProvider::new(ProviderId::LM_STUDIO, "LM Studio", base_url).with_quirks(
        ProviderQuirks {
            overflow_patterns: vec![
                "greater than the context length".to_string(),
            ],
            ..Default::default()
        },
    )
}

/// llama.cpp — lightweight C++ inference server.
/// Reads `LLAMA_CPP_HOST` for the base URL; defaults to `http://localhost:8080`.
pub fn llama_cpp() -> OpenAiCompatProvider {
    let host = std::env::var("LLAMA_CPP_HOST")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    let base_url = format!("{}/v1", host.trim_end_matches('/'));
    OpenAiCompatProvider::new(ProviderId::LLAMA_CPP, "llama.cpp", base_url).with_quirks(
        ProviderQuirks {
            overflow_patterns: vec![
                "exceeds the available context size".to_string(),
            ],
            ..Default::default()
        },
    )
}

