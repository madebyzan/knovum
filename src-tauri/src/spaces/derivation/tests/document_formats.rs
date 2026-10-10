use crate::documents::generation::{document::DocumentSource, pdf};

#[test]
fn conversion_title_uses_the_filename_stem_only_for_automatic_file_titles() {
    let title = super::super::document_formats::resolve_source_title;
    assert_eq!(
        title(
            "file",
            &serde_json::json!({"title":"Report.docx", "fileName":"Report.docx"})
        )
        .unwrap(),
        "Report"
    );
    assert_eq!(
        title(
            "file",
            &serde_json::json!({"title":"Custom title", "fileName":"Report.docx"})
        )
        .unwrap(),
        "Custom title"
    );
    assert_eq!(
        title("markdown", &serde_json::json!({"title":"Release v1.2"})).unwrap(),
        "Release v1.2"
    );
}

#[test]
fn html_conversion_uses_the_local_document_format_and_file_contract() {
    let target = super::super::document_formats::require("convert-document-to-html").unwrap();
    assert_eq!(target.extension, "html");
    let action = crate::spaces::actions::require("convert-document-to-html").unwrap();
    assert!(action.execution_capability.is_none());
    assert!(action.routing_purpose.is_none());
    assert!(matches!(
        action.output_contract,
        crate::spaces::actions::SpaceActionOutputContract::File
    ));
    assert!(crate::spaces::actions::validate_parameters(action, &serde_json::json!({})).is_ok());
}

#[tokio::test]
async fn conversion_actions_match_the_current_derivation_baseline() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.unwrap();
    for extension in ["docx", "pdf", "txt", "html"] {
        let action_id = format!("convert-document-to-{extension}");
        let target = super::super::document_formats::require(&action_id).unwrap();
        assert_eq!(target.extension, extension);
        crate::spaces::actions::require(&action_id).unwrap();
        assert!(
            crate::spaces::actions::require(&format!("export-document-as-{extension}")).is_err()
        );
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,data,initial_x,initial_y,created_at,updated_at) VALUES($1,'space','file','{\"kind\":\"file\",\"title\":\"Report\"}',0,0,1,1)")
            .bind(extension).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,'space',$2,$1,1,'test','running',1,1)")
            .bind(action_id).bind(extension).execute(&pool).await.unwrap();
    }
}

#[test]
fn typst_source_preserves_headings_lists_and_unicode() {
    let source = pdf::tests::render_source(&DocumentSource {
        markdown: "# 标题\n\n- 第一项\n- **Second** item".into(),
        ..Default::default()
    })
    .expect("generate Typst source");
    assert!(source.contains("#heading(level: 1)"));
    assert!(source.contains("第一项"));
    assert!(source.contains("#strong[#text(\"Second\")]"));
}

#[test]
fn renders_a_valid_pdf_with_platform_cjk_font() {
    let bytes = pdf::render(&DocumentSource {
        title: "知识文档".into(),
        markdown: "Hello, 世界。".into(),
        ..Default::default()
    })
    .expect("render PDF");
    assert!(bytes.starts_with(b"%PDF-"));
}
