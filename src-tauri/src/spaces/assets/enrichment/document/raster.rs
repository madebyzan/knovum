use super::resource;
use crate::spaces::knowledge::node_contracts::document_coverage::{
    DocumentCoverage, DocumentResource, MAX_TOTAL_RESOURCE_BYTES,
};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(super) fn materialize_all(
    app: &AppHandle,
    source: &std::path::Path,
    report: &mut DocumentCoverage,
    cancellation: &CancellationToken,
    claim_managed_path: &mut dyn FnMut(&std::path::Path) -> Result<(), String>,
) -> Result<(), String> {
    for index in 0..report.occurrences.len() {
        crate::spaces::assets::enrichment::ensure_active(cancellation)?;
        if report.occurrences[index].resource_id.is_some() {
            continue;
        }
        let input = report.occurrences[index]
            .pdf_input
            .as_ref()
            .ok_or("Document occurrence has no PDF rendering input")?;
        let resource = resource::save_render(app, source, input, claim_managed_path)?;
        attach_resource(report, index, resource)?;
    }
    Ok(())
}

fn attach_resource(
    report: &mut DocumentCoverage,
    index: usize,
    resource: DocumentResource,
) -> Result<(), String> {
    let new_resource = !report.resources.iter().any(|value| value.id == resource.id);
    if new_resource {
        let bytes = report
            .resources
            .iter()
            .map(|value| value.byte_size)
            .sum::<u64>()
            + resource.byte_size;
        if bytes > MAX_TOTAL_RESOURCE_BYTES {
            return Err("Document visual resources exceed 256 MiB".into());
        }
    }
    report.occurrences[index].resource_id = Some(resource.id.clone());
    if new_resource {
        report.resources.push(resource);
    }
    Ok(())
}
