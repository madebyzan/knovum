use super::endpoint;

#[test]
fn image_and_video_share_minimax_endpoint_normalization() {
    assert_eq!(endpoint(""), "https://api.minimax.cn/v1");
    assert_eq!(
        endpoint("https://api.minimax.cn/"),
        "https://api.minimax.cn/v1"
    );
    assert_eq!(
        endpoint("https://api.minimax.cn/v1/"),
        "https://api.minimax.cn/v1"
    );
}
