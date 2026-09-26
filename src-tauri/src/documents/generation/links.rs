/// Export never creates links capable of opening local files or executing a custom protocol.
pub(super) fn validate(url: &str) -> Result<(), String> {
    let scheme = url.split_once(':').map(|(scheme, _)| scheme);
    if scheme.is_some_and(|scheme| {
        ["https", "http", "mailto"]
            .iter()
            .any(|allowed| scheme.eq_ignore_ascii_case(allowed))
    }) && !url.chars().any(char::is_control)
    {
        Ok(())
    } else {
        Err("Document hyperlinks must use HTTPS, HTTP or mailto; relative links are not yet supported".into())
    }
}
