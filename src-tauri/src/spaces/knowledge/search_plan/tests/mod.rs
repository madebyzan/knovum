use super::build;

#[test]
fn uses_substring_for_short_tokens_and_fts_for_longer_terms() {
    let short = build("AI 搜索").expect("plan");
    assert_eq!(short.terms, ["AI", "搜索"]);
    assert!(short.fts_query.is_none());
    let long = build("apple design").expect("plan");
    assert_eq!(long.fts_query.as_deref(), Some("\"apple\" AND \"design\""));
}

#[test]
fn strips_only_outer_quote_punctuation() {
    let plan = build("“alpha” 'beta'").expect("plan");
    assert_eq!(plan.terms, ["alpha", "beta"]);
}

#[test]
fn decomposes_cjk_sentences_into_searchable_subterms() {
    let plan = build("如何管理知识").expect("plan");
    assert_eq!(plan.terms, ["如何", "管理", "知识"]);
    assert_eq!(plan.fts_query.as_deref(), Some("\"如何管理知识\""));

    let odd = build("知识库管理").expect("plan");
    assert_eq!(odd.terms, ["知识", "库管理"]);
}
