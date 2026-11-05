use super::unsupported_output;

#[test]
fn identifies_unsupported_yt_dlp_outputs() {
    assert!(unsupported_output("ERROR: Unsupported URL: https://example.com"));
    assert!(unsupported_output(
        "ERROR: [twitter] 2117962852107686109: No video could be found in this tweet"
    ));
    assert!(unsupported_output("ERROR: No video could be found in this webpage"));
    assert!(unsupported_output("ERROR: There's no video in this post"));
    assert!(!unsupported_output(
        "ERROR: Unable to download webpage: HTTP Error 403: Forbidden"
    ));
}
