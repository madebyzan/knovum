use super::{audio_error, classify_audio_error};
use crate::dictation_contracts::DictationFailure;

#[test]
fn permission_denial_uses_the_error_kind_even_with_a_localized_message() {
    let error = cpal::Error::with_message(cpal::ErrorKind::PermissionDenied, "麦克风访问被拒绝");

    let DictationFailure::Classified { code, message } = classify_audio_error(error) else {
        panic!("permission denial must be classified")
    };
    assert_eq!(code, "microphone_permission_denied");
    assert_eq!(message, "麦克风访问被拒绝");
}

#[test]
fn device_unavailability_does_not_infer_permission_denial_from_message_text() {
    let error = cpal::Error::with_message(
        cpal::ErrorKind::DeviceNotAvailable,
        "No access to a disconnected device",
    );

    let DictationFailure::Classified { code, message } = audio_error(&error) else {
        panic!("device failure must be classified")
    };
    assert_eq!(code, "microphone_unavailable");
    assert_eq!(message, "No access to a disconnected device");
}
