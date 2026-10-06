use super::secret_name;

#[test]
fn credential_names_are_rejected_without_blocking_token_counts_or_tokenizer_options() {
    for field in [
        "max_tokens",
        "token_limit",
        "TOKENIZER_PARALLELISM",
        "secretary",
        "password_length",
    ] {
        assert!(!secret_name(field), "ordinary field rejected: {field}");
    }
    for field in [
        "Authorization",
        "X-Api-Key",
        "OPENAI_API_KEY",
        "access_token",
        "clientSecret",
        "AWS_SECRET_ACCESS_KEY",
        "private_key",
    ] {
        assert!(secret_name(field), "credential field accepted: {field}");
    }
}
