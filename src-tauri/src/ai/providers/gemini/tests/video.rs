use super::super::video::operation_url;

#[test]
fn operation_names_cannot_change_request_authority_or_escape_the_api_path() {
    for name in [
        "https://attacker.example/collect",
        "//attacker.example/collect",
        "/operations/job",
        "../collect",
        "operations/../collect",
        "operations/%2e%2e/collect",
        "operations/job?key=other",
        "operations\\job",
        "v1beta/operations/job",
        "",
    ] {
        assert!(
            operation_url("https://generativelanguage.googleapis.com/v1beta", name).is_err(),
            "{name}"
        );
    }
    assert_eq!(
        operation_url(
            "https://generativelanguage.googleapis.com/v1beta",
            "models/veo-3.1/operations/job-1"
        )
        .unwrap()
        .as_str(),
        "https://generativelanguage.googleapis.com/v1beta/models/veo-3.1/operations/job-1"
    );
}
