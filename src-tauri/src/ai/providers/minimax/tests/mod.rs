use super::endpoint;

#[test]
fn image_and_video_share_minimax_endpoint_normalization() {
    assert_eq!(endpoint(""), "https://api.minimax.chat/v1");
    assert_eq!(
        endpoint("https://api.minimax.chat/"),
        "https://api.minimax.chat/v1"
    );
    assert_eq!(
        endpoint("https://api.minimax.chat/v1/"),
        "https://api.minimax.chat/v1"
    );
}
