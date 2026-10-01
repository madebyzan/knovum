use super::{media_acquisition_error, media_preparation_error};

#[test]
fn media_cancellation_has_an_explicit_command_code() {
    let acquisition = media_acquisition_error(
        crate::media::acquisition::yt_dlp::ACQUISITION_CANCELLED.to_owned(),
    );
    let preparation = media_preparation_error(
        crate::local_resources::preparation::RESOURCE_PREPARATION_CANCELLED.to_owned(),
    );
    assert_eq!(acquisition.code, "media_acquisition_cancelled");
    assert_eq!(preparation.code, "media_acquisition_cancelled");
    assert!(preparation.message.contains("Media import was cancelled"));
}
