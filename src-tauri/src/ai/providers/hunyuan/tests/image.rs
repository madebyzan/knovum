use super::*;

#[test]
fn every_supported_ratio_maps_to_a_size_within_the_api_limits() {
    for ratio in ["16:9", "9:16", "4:3", "3:4", "1:1"] {
        let size = size_for_ratio(ratio);
        let (width, height) = size.split_once('x').expect("WxH size");
        let (width, height) = (
            width.parse::<u32>().expect("width"),
            height.parse::<u32>().expect("height"),
        );
        assert!((512..=2048).contains(&width), "{ratio} -> {size}");
        assert!((512..=2048).contains(&height), "{ratio} -> {size}");
        assert!(width * height <= 1024 * 1024, "{ratio} -> {size}");
    }
    assert_eq!(size_for_ratio("21:9"), "1024x1024");
}

#[test]
fn a_completed_response_exposes_the_temporary_image_url() {
    let parsed: ImageResponse = serde_json::from_value(json!({
        "id": "4-WandImage-a786",
        "created": 1785125529,
        "data": [{ "url": "https://aigc-image.cos.myqcloud.com/x/result.png", "revised_prompt": "rewritten" }],
    }))
    .expect("valid image payload");
    assert_eq!(
        parsed.data[0].url.as_deref(),
        Some("https://aigc-image.cos.myqcloud.com/x/result.png")
    );
    assert_eq!(parsed.data[0].revised_prompt.as_deref(), Some("rewritten"));
}

#[test]
fn an_empty_data_list_deserializes_so_the_caller_can_reject_it() {
    let parsed: ImageResponse = serde_json::from_value(json!({ "data": [] })).expect("empty data");
    assert!(parsed.data.is_empty());
    let missing: ImageResponse = serde_json::from_value(json!({})).expect("absent data");
    assert!(missing.data.is_empty());
}
