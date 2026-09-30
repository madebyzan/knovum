use super::*;

fn target(provider: &str, model: &str) -> ModelExecutionTarget {
    ModelExecutionTarget {
        provider: provider.into(),
        model: model.into(),
        base_url: String::new(),
        temperature: 0.0,
        secret_key: String::new(),
    }
}

fn operations(provider: &str, model: &str) -> SupportedOperations {
    supported_operations(backend(&target(provider, model)))
}

#[test]
fn image_operation_routes_match_capabilities() {
    assert_eq!(
        backend(&target("qwen", "wanx2.1-imageedit")),
        ImageOperationBackend::DashScope
    );
    let qwen = operations("qwen", "wanx2.1-imageedit");
    assert!(qwen.upscale && qwen.multi_angle && qwen.masked_removal);
    assert!(!qwen.prompt_removal);

    assert_eq!(
        backend(&target("openai", "gpt-image-1.5")),
        ImageOperationBackend::OpenAi
    );
    let openai = operations("openai", "gpt-image-1.5");
    assert!(openai.masked_removal);
    assert!(!openai.upscale && !openai.multi_angle && !openai.prompt_removal);

    assert_eq!(
        backend(&target("gemini", "gemini-2.5-flash-image")),
        ImageOperationBackend::Gemini
    );
    let gemini = operations("gemini", "gemini-2.5-flash-image");
    assert!(gemini.prompt_removal);
    assert!(!gemini.upscale && !gemini.multi_angle && !gemini.masked_removal);

    for (provider, model, expected) in [
        (
            "grok",
            "grok-imagine-image-2.0",
            ImageOperationBackend::ImageEdit(ImageEditApi::Grok),
        ),
        (
            "doubao",
            "doubao-seedream-4-5-250528",
            ImageOperationBackend::ImageEdit(ImageEditApi::Doubao),
        ),
        (
            "openrouter",
            "openai/gpt-image-1",
            ImageOperationBackend::ImageEdit(ImageEditApi::OpenRouter),
        ),
        (
            "openrouter",
            "google/gemini-2.5-flash-image",
            ImageOperationBackend::ImageEdit(ImageEditApi::OpenRouter),
        ),
        (
            "openrouter",
            "bytedance-seed/seedream-4.5",
            ImageOperationBackend::ImageEdit(ImageEditApi::OpenRouter),
        ),
        (
            "openrouter",
            "black-forest-labs/flux.2-pro",
            ImageOperationBackend::ImageEdit(ImageEditApi::OpenRouter),
        ),
        (
            "siliconflow",
            "Qwen/Qwen-Image-Edit-2509",
            ImageOperationBackend::ImageEdit(ImageEditApi::SiliconFlow),
        ),
        ("kling", "kling-image-o1", ImageOperationBackend::Kling),
        ("kling", "kling-v3-omni", ImageOperationBackend::Kling),
    ] {
        assert_eq!(backend(&target(provider, model)), expected);
        let supported = operations(provider, model);
        assert!(supported.prompt_removal, "{provider}: {model}");
        assert!(!supported.masked_removal, "{provider}: {model}");
    }

    let unsupported = operations("openai-compatible", "gpt-image-1.5");
    assert!(!unsupported.masked_removal && !unsupported.prompt_removal);
    assert_eq!(
        backend(&target("openai-compatible", "gpt-image-1.5")),
        ImageOperationBackend::Unsupported
    );
    for (provider, model) in [
        ("grok", "grok-4"),
        ("doubao", "doubao-seed-1-6"),
        ("openrouter", "anthropic/claude-sonnet-4"),
        ("siliconflow", "Kwai-Kolors/Kolors"),
        ("kling", "kling-v2-1"),
    ] {
        assert!(!operations(provider, model).prompt_removal);
    }
}
