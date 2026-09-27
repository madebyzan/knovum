use super::contracts::PackageChunk;

pub fn validate_packaged_derived(
    packaged: &[&PackageChunk],
    generated: &[crate::spaces::knowledge::indexing::KnowledgeChunkSeed],
) -> Result<(), String> {
    let mut actual = packaged
        .iter()
        .map(|row| {
            serde_json::json!({
                "kind": row.kind,
                "text": row.text,
                "locator": row.locator,
                "metadata": row.metadata,
            })
            .to_string()
        })
        .collect::<Vec<_>>();
    let mut expected = generated
        .iter()
        .map(|row| {
            serde_json::json!({
                "kind": row.kind,
                "text": row.text,
                "locator": row.locator,
                "metadata": row.metadata,
            })
            .to_string()
        })
        .collect::<Vec<_>>();
    actual.sort();
    expected.sort();
    if actual == expected {
        Ok(())
    } else {
        Err("Package derived Knowledge passages do not match authoritative node content".into())
    }
}
