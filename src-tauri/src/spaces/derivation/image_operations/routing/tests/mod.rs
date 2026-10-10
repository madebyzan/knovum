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
    assert!(qwen.multi_angle);
    assert!(!qwen.prompt_removal);

    assert_eq!(
        backend(&target("openai", "gpt-image-1.5")),
        ImageOperationBackend::Unsupported
    );
    assert_eq!(
        backend(&target("gemini", "gemini-2.5-flash-image")),
        ImageOperationBackend::Gemini
    );
    let gemini = operations("gemini", "gemini-2.5-flash-image");
    assert!(gemini.prompt_removal);
    assert!(!gemini.multi_angle);

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
        assert!(!supported.multi_angle, "{provider}: {model}");
    }

    let unsupported = operations("openai-compatible", "gpt-image-1.5");
    assert!(!unsupported.multi_angle && !unsupported.prompt_removal);
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

#[test]
fn remote_routes_do_not_advertise_local_processing() {
    for (provider, model) in [
        ("qwen", "wanx2.1-imageedit"),
        ("gemini", "gemini-2.5-flash-image"),
        ("openai", "gpt-image-1.5"),
        ("kling", "kling-image-o1"),
    ] {
        let supported = operations(provider, model);
        for operation in [
            ImageOperation::Upscale {
                target_long_edge: 2048,
            },
            ImageOperation::RemoveObject { strokes: vec![] },
            ImageOperation::RemoveBackground,
            ImageOperation::GenerateDepth,
        ] {
            assert!(!supported.allows(&operation), "{provider}: {model}");
        }
    }
}
