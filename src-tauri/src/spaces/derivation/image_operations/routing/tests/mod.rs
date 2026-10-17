use super::*;

fn target(provider: &str, model: &str) -> ModelExecutionTarget {
    ModelExecutionTarget {
        capabilities: vec![crate::ai::contracts::ExecutionCapability::TextGeneration],
        provider: provider.into(),
        model: model.into(),
        base_url: String::new(),
        temperature: 0.0,
        secret_key: String::new(),
    }
}

#[test]
fn remote_operations_follow_the_models_declared_reference_support() {
    for (provider, model) in [
        ("qwen", "wanx2.1-imageedit"),
        ("gemini", "gemini-2.5-flash-image"),
        ("gemini", "gemini-3-pro-image"),
        ("grok", "grok-imagine-image-2.0"),
        ("doubao", "doubao-seedream-4-5-250528"),
        ("openrouter", "openai/gpt-image-1"),
        ("openrouter", "google/gemini-2.5-flash-image"),
        ("openrouter", "bytedance-seed/seedream-4.5"),
        ("openrouter", "black-forest-labs/flux.2-pro"),
        ("siliconflow", "Qwen/Qwen-Image-Edit-2509"),
        ("kling", "kling-image-o1"),
        ("kling", "kling-v3-omni"),
    ] {
        assert!(
            supports_remote_operations(&target(provider, model)),
            "{provider}: {model} accepts a reference image"
        );
    }
}

#[test]
fn a_text_only_image_model_cannot_run_a_remote_operation() {
    for (provider, model) in [
        ("openai", "gpt-image-1.5"),
        ("openai-compatible", "gpt-image-1.5"),
        ("gemini", "imagen-4.0-generate-001"),
        ("grok", "grok-4"),
        ("doubao", "doubao-seed-1-6"),
        ("openrouter", "anthropic/claude-sonnet-4"),
        ("siliconflow", "Kwai-Kolors/Kolors"),
        ("kling", "kling-v2-1"),
        ("anthropic", "claude-sonnet-4"),
    ] {
        assert!(
            !supports_remote_operations(&target(provider, model)),
            "{provider}: {model} has no verified reference-image API"
        );
    }
}
