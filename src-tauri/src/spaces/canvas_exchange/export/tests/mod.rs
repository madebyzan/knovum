use super::*;
#[test]
fn publishes_shared_file_and_group_background_reference() {
    let directory =
        std::env::temp_dir().join(format!("canvas-export-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let source = directory.join("image.png");
    std::fs::write(&source, b"image bytes").unwrap();
    let mut document = serde_json::from_value(serde_json::json!({"nodes":[
            {"id":"file","type":"file","x":0,"y":0,"width":200,"height":100,"file":"assets/image.png"},
            {"id":"group","type":"group","x":0,"y":0,"width":400,"height":300,"background":"assets/image.png"}
        ]})).unwrap();
    let destination = directory.join("export.canvas");
    publish(
        &destination,
        &directory,
        &mut document,
        &[(source, "assets/image.png".into())],
    )
    .unwrap();
    let publication: serde_json::Value =
        serde_json::from_slice(&std::fs::read(destination).unwrap()).unwrap();
    assert_eq!(
        publication["nodes"][0]["file"],
        publication["nodes"][1]["background"]
    );
    assert_eq!(
        std::fs::read(directory.join(publication["nodes"][0]["file"].as_str().unwrap())).unwrap(),
        b"image bytes"
    );
    std::fs::remove_dir_all(directory).unwrap();
}
