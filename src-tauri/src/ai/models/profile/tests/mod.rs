use super::{catalog::Catalog, ModelProfile};
use crate::ai::contracts::ProviderKind;

const PUBLISHED: &str = r#"{
  "openai": { "id": "openai", "models": {
    "gpt-5": { "limit": { "context": 400000, "input": 272000, "output": 128000 } },
    "gpt-4o": { "limit": { "context": 128000, "output": 16384 } }
  } },
  "alibaba-cn": { "id": "alibaba-cn", "models": {
    "qwen-plus": { "limit": { "context": 1000000, "output": 32768 } }
  } },
  "alibaba": { "id": "alibaba", "models": {
    "qwen-plus": { "limit": { "context": 131072, "output": 8192 } },
    "qwen-max": { "limit": { "context": 32768, "output": 8192 } }
  } },
  "reseller": { "id": "reseller", "models": {
    "gpt-4o": { "limit": { "context": 64000, "output": 32000 } },
    "unknown-limits": { "limit": { "context": 0, "output": 0 } },
    "no-limit": {}
  } },
  "empty": { "id": "empty", "models": {} }
}"#;

fn catalog() -> Catalog {
    Catalog::parse(PUBLISHED.as_bytes(), 0).expect("published catalog parses")
}

fn limits(input: u32, output: u32) -> ModelProfile {
    ModelProfile {
        max_input_tokens: Some(input),
        max_output_tokens: Some(output),
    }
}

#[test]
fn a_published_input_limit_takes_precedence_over_the_context_window() {
    assert_eq!(
        catalog().profile(ProviderKind::OpenAi, "gpt-5"),
        limits(272_000, 128_000)
    );
    assert_eq!(
        catalog().profile(ProviderKind::OpenAi, "gpt-4o"),
        limits(128_000, 16_384)
    );
}

#[test]
fn a_provider_reads_its_own_listings_in_order() {
    assert_eq!(
        catalog().profile(ProviderKind::DashScope, "qwen-plus"),
        limits(1_000_000, 32_768)
    );
    assert_eq!(
        catalog().profile(ProviderKind::DashScope, "qwen-max"),
        limits(32_768, 8_192)
    );
    assert_eq!(
        catalog().profile(ProviderKind::Anthropic, "gpt-4o"),
        ModelProfile::default()
    );
}

#[test]
fn a_generic_endpoint_applies_the_limits_every_listing_honours() {
    assert_eq!(
        catalog().profile(ProviderKind::OpenAiCompatible, "gpt-4o"),
        limits(64_000, 16_384)
    );
}

#[test]
fn unpublished_limits_stay_unknown() {
    let catalog = catalog();
    for model in ["unknown-limits", "no-limit", "never-listed"] {
        assert_eq!(
            catalog.profile(ProviderKind::OpenAiCompatible, model),
            ModelProfile::default()
        );
    }
    assert_eq!(
        catalog.profile(ProviderKind::Ollama, "qwen-plus"),
        ModelProfile::default()
    );
}

#[test]
fn a_catalog_without_any_limit_is_rejected() {
    assert!(Catalog::parse(br#"{"empty": {"models": {}}}"#, 0).is_err());
    assert!(Catalog::parse(b"[]", 0).is_err());
}
