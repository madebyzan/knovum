use super::{alignment, embedding, image, moss_nano, threaded, whisper, WHISPER_VARIANTS};
use crate::runtime::RuntimeCapability;

/// The identities are a cross-process handshake: pin them so a rename has to be deliberate.
#[test]
fn bundled_profile_identities_are_frozen() {
    assert_eq!(threaded("cpu-f32", 4), "cpu-f32-threads4-bundled-v2");
    assert_eq!(embedding("mps", 6), "mps-f32-threads6-bundled-v2");
    assert_eq!(moss_nano(1), "moss-nano-cpu-f32-threads1-bundled-v2");
    assert_eq!(
        whisper("large-v3", "cpu", "f16"),
        "whisper-large-v3-cpu-f16-bundled-v2"
    );
    assert_eq!(
        image(RuntimeCapability::ImageUpscaling, 2),
        Some("image-upscaling-cpu-threads2-bundled-v2".to_owned())
    );
    assert_eq!(
        image(RuntimeCapability::ImageSegmentation, 2),
        Some(embedding("cpu", 2))
    );
}

#[test]
fn whisper_variants_cover_the_published_precisions() {
    assert_eq!(
        WHISPER_VARIANTS.map(|(model, precision)| whisper(model, "mps", precision)),
        [
            "whisper-tiny-mps-q8_0-bundled-v2",
            "whisper-small-mps-q8_0-bundled-v2",
            "whisper-large-v3-mps-f16-bundled-v2"
        ]
    );
}

#[test]
fn non_image_capabilities_have_no_image_profile() {
    for capability in [
        RuntimeCapability::Embedding,
        RuntimeCapability::SpeechTranscription,
        RuntimeCapability::SpeechSynthesis,
    ] {
        assert!(image(capability, 4).is_none());
    }
}

#[test]
fn alignment_identities_do_not_reinterpret_obsolete_values() {
    for (spec, precision) in [
        ("qwen3-forced-aligner-0.6b-f16-bundled-v2", "f16"),
        ("qwen3-forced-aligner-0.6b-q8_0-v1", "q8_0"),
    ] {
        for device in ["cpu", "mps", "vulkan"] {
            assert_eq!(
                alignment(spec, device).unwrap(),
                format!("qwen3-forced-aligner-0.6b-{precision}-{device}-bundled-v2")
            );
        }
        assert!(alignment(spec, "cuda").is_none());
    }
    assert!(alignment("qwen3-forced-aligner-0.6b-v1", "cpu").is_none());
}
