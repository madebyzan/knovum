use super::parse;

#[test]
fn standard_skill_names_support_unicode_and_count_characters() {
    for name in ["研究笔记", "étude", &"研".repeat(64)] {
        let markdown = format!("---\nname: {name}\ndescription: Research notes\n---\n# Research");
        assert_eq!(parse(markdown.as_bytes(), name).unwrap().name, name);
    }
    for name in ["Étude", "research--notes", &"研".repeat(65)] {
        let markdown = format!("---\nname: {name}\ndescription: Research notes\n---\n# Research");
        assert!(parse(markdown.as_bytes(), name).is_err());
    }
}

#[test]
fn compatibility_has_the_standard_character_budget() {
    for length in [1, 500, 501] {
        let markdown = format!(
            "---\nname: research\ndescription: Research notes\ncompatibility: {}\n---\n# Research",
            "研".repeat(length)
        );
        assert_eq!(
            parse(markdown.as_bytes(), "research").is_ok(),
            length <= 500
        );
    }
    assert!(parse(
        b"---\nname: research\ndescription: Research notes\ncompatibility: ''\n---\n",
        "research"
    )
    .is_err());
}
