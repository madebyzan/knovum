use super::{parse_json_canvas, serialize_json_canvas};

#[test]
fn accepts_the_json_canvas_1_0_contract() {
    let source = r##"{"nodes":[{"id":"a","type":"text","x":0,"y":1,"width":200,"height":100,"text":"Hello"},{"id":"b","type":"link","x":300,"y":1,"width":200,"height":100,"url":"https://example.com","color":"#AABBCC"}],"edges":[{"id":"e","fromNode":"a","toNode":"b"}]}"##;
    let document = parse_json_canvas(source.into()).expect("valid JSON Canvas document");
    let encoded = serialize_json_canvas(document).expect("serializable JSON Canvas document");
    assert!(encoded.contains("\"fromNode\": \"a\""));
}

#[test]
fn preserves_extension_fields_and_rejects_dangling_edges() {
    let extension = r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":1,"height":1,"text":"x","vendor":true}]}"#;
    let document = parse_json_canvas(extension.into()).expect("extension fields are allowed");
    let encoded = serialize_json_canvas(document).expect("extension fields are preserved");
    assert!(encoded.contains("\"vendor\": true"));
    let dangling = r#"{"edges":[{"id":"e","fromNode":"missing","toNode":"missing"}]}"#;
    assert_eq!(
        parse_json_canvas(dangling.into()).unwrap_err().message,
        "edges[0].fromNode does not reference a node"
    );
}

#[test]
fn rejects_non_integer_geometry_and_invalid_colors() {
    let geometry =
        r#"{"nodes":[{"id":"a","type":"text","x":0.5,"y":0,"width":1,"height":1,"text":"x"}]}"#;
    assert_eq!(
        parse_json_canvas(geometry.into()).unwrap_err().message,
        "nodes[0].x must be an integer"
    );
    let unsafe_geometry = r#"{"nodes":[{"id":"a","type":"text","x":9007199254740992,"y":0,"width":1,"height":1,"text":"x"}]}"#;
    assert_eq!(
        parse_json_canvas(unsafe_geometry.into())
            .unwrap_err()
            .message,
        "nodes[0].x must be a safe integer"
    );
    let zero_width =
        r#"{"nodes":[{"id":"a","type":"text","x":0,"y":0,"width":0,"height":1,"text":"x"}]}"#;
    assert_eq!(
        parse_json_canvas(zero_width.into()).unwrap_err().message,
        "nodes[0] width and height must be positive integers"
    );
    for color in ["red", "#fff"] {
        let source = format!(
            r##"{{"nodes":[{{"id":"a","type":"text","x":0,"y":0,"width":1,"height":1,"text":"x","color":"{color}"}}]}}"##
        );
        assert_eq!(
            parse_json_canvas(source).unwrap_err().message,
            "nodes[0].color must be a JSON Canvas preset or hexadecimal color"
        );
    }
}
mod import;
