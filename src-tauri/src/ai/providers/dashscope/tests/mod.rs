use super::{endpoint, protocol};

#[test]
fn all_dashscope_adapters_share_the_same_api_endpoint() {
    assert_eq!(
        endpoint("https://dashscope.aliyuncs.com/api/v1/"),
        "https://dashscope.aliyuncs.com/api/v1"
    );
    assert_eq!(
        endpoint("https://dashscope.aliyuncs.com/compatible-mode/v1"),
        "https://dashscope.aliyuncs.com/api/v1"
    );
}

#[test]
fn wan_request_serializes_reference_images_in_selection_order() {
    let references = vec![
        "data:image/png;base64,one".to_owned(),
        "data:image/jpeg;base64,two".to_owned(),
    ];
    let request = protocol::create_wan_request(
        "wan3.0-video",
        "Use Image 1 then Image 2",
        "16:9",
        5,
        &references,
    );
    let value = serde_json::to_value(request).expect("request serializes");
    assert_eq!(value["input"]["media"][0]["type"], "reference_image");
    assert_eq!(value["input"]["media"][1]["url"], references[1]);
}
