use super::*;

fn texts(markdown: &str) -> Vec<(String, Vec<u32>)> {
    segment(markdown)
        .statements
        .into_iter()
        .map(|statement| (statement.text, statement.citations))
        .collect()
}

#[test]
fn a_marker_cites_the_sentence_it_follows() {
    assert_eq!(
        texts("Revenue grew 12% in Q3. [C2] Costs were flat [C1][C3]. Margins rose."),
        vec![
            ("Revenue grew 12% in Q3.".into(), vec![2]),
            ("Costs were flat.".into(), vec![1, 3]),
            ("Margins rose.".into(), vec![]),
        ]
    );
}

#[test]
fn ranges_point_at_the_sentence_in_the_markdown_source() {
    let markdown = "Intro line here.\n\n- **Fast** start up [C1]\n- Lower cost overall";
    let statements = segment(markdown).statements;
    assert_eq!(statements.len(), 3);
    assert_eq!(
        &markdown[statements[1].start..statements[1].end],
        "Fast** start up"
    );
    assert_eq!(statements[1].citations, vec![1]);
    assert_eq!(
        &markdown[statements[2].start..statements[2].end],
        "Lower cost overall"
    );
}

#[test]
fn headings_code_and_table_headers_are_not_statements() {
    let markdown = "# Findings overview\n\n```\nlet value = compute();\n```\n\n| Option | Price |\n|---|---|\n| Plan A | Costs ten dollars [C4] |\n";
    assert_eq!(
        texts(markdown),
        vec![
            ("Plan A".into(), vec![]),
            ("Costs ten dollars".into(), vec![4])
        ]
    );
}

#[test]
fn chinese_sentences_split_on_their_own_terminators() {
    assert_eq!(
        texts("方案A的成本更低。[C1]部署也更快。"),
        vec![
            ("方案A的成本更低。".into(), vec![1]),
            ("部署也更快。".into(), vec![])
        ]
    );
}

#[test]
fn ids_count_statements_in_order() {
    let statements = segment("First claim here. Second claim here.").statements;
    assert_eq!(
        statements.iter().map(|s| s.id).collect::<Vec<_>>(),
        vec![1, 2]
    );
}

#[test]
fn a_long_answer_is_cut_at_the_word_budget() {
    let sentence = "This sentence has exactly eight short words. ";
    let segmentation = segment(&sentence.repeat(MAX_WORDS / 8 + 10));
    assert!(segmentation.truncated);
    assert!(segmentation.statements.len() <= MAX_WORDS / 8);
}

#[test]
fn marker_parsing_accepts_only_well_formed_markers() {
    assert_eq!(marker_at("[C12] rest"), Some((12, 5)));
    assert_eq!(marker_at("[C0]"), None);
    assert_eq!(marker_at("[C]"), None);
    assert_eq!(marker_at("[C3"), None);
}
