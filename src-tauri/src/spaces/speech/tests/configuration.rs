use serde_json::json;

#[test]
fn output_configuration_rejects_wrong_shapes_obsolete_fields_and_invalid_bounds() {
    use crate::spaces::speech::project_config::parse;
    assert!(parse(
        "short_dubbing",
        &json!({"format":"wav","wavPrecision":"pcm24"})
    )
    .is_ok());
    for value in [
        json!({"format":"mp3","wavPrecision":"pcm24"}),
        json!({"format":"wav"}),
        json!({"format":"wav","wavPrecision":"pcm24","legacy":true}),
    ] {
        assert!(parse("short_dubbing", &value).is_err());
    }
    let mut book = json!({
        "format":"m4b","sampleRate":48000,"loudnessLufs":-20,"truePeakDb":-2,
        "chapters":[],"roles":[],"paragraphPauseMs":250,"chapterPauseMs":1000,
        "wavPrecision":"pcm16","metadata":{"title":"Book","author":"","narrator":"","cover":null}
    });
    assert!(parse("audiobook", &book).is_ok());
    assert!(parse("short_dubbing", &book).is_err());
    book["loudnessLufs"] = json!(-40);
    assert!(parse("audiobook", &book).is_err());
    assert!(parse("obsolete", &json!({})).is_err());
}

async fn project_database(output: &str, lexicon: &str) -> sqlx::SqlitePool {
    let pool = super::database().await;
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space','note',0,0,'{\"kind\":\"note\",\"title\":\"Source\"}',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_speech_projects(id,space_id,source_node_id,project_type,title,created_at,updated_at) VALUES('project','space','node','short_dubbing','Draft',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_speech_project_revisions(id,project_id,revision,source_revision,pronunciation_lexicon,output_config,created_at) VALUES('revision','project',1,'source',?,?,1)")
        .bind(lexicon).bind(output).execute(&pool).await.unwrap();
    sqlx::query(
        "UPDATE spaces_speech_projects SET active_revision_id='revision' WHERE id='project'",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

#[tokio::test]
async fn project_response_contains_structured_configuration_and_lexicon() {
    let config = json!({"format":"wav","wavPrecision":"pcm24"});
    let lexicon = json!({"Knovum":"Knowledge"});
    let pool = project_database(&config.to_string(), &lexicon.to_string()).await;
    let project = crate::spaces::speech::repository::load_project(&pool, "project")
        .await
        .unwrap();
    let response = serde_json::to_value(project).unwrap();
    assert_eq!(response["revision"]["outputConfig"], config);
    assert_eq!(response["revision"]["pronunciationLexicon"], lexicon);
}

#[tokio::test]
async fn project_response_rejects_invalid_persisted_configuration_and_lexicon() {
    for (config, lexicon) in [
        (r#"{"format":"mp3","wavPrecision":"pcm16"}"#, "{}"),
        (
            r#"{"format":"wav","wavPrecision":"pcm16","legacy":true}"#,
            "{}",
        ),
        (
            r#"{"format":"wav","wavPrecision":"pcm16"}"#,
            r#"{" word ":"sound"}"#,
        ),
        (
            r#"{"format":"wav","wavPrecision":"pcm16"}"#,
            r#"{"word":42}"#,
        ),
    ] {
        let pool = project_database(config, lexicon).await;
        assert!(
            crate::spaces::speech::repository::load_project(&pool, "project")
                .await
                .is_err()
        );
    }
}
