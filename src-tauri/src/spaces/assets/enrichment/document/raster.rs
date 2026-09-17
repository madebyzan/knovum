use super::{model::DocumentCoverage, resource};
use tauri::AppHandle;

pub(super) async fn prepare_raster(
    app: &AppHandle,
    source: &std::path::Path,
    report: &mut DocumentCoverage,
    index: usize,
) -> Result<(), String> {
    let input = report.occurrences[index]
        .pdf_input
        .clone()
        .ok_or("Document occurrence has no PDF rendering input")?;
    let app = app.clone();
    let source = source.to_owned();
    let resource =
        crate::blocking::run(move || resource::save_render(&app, &source, &input)).await?;
    let new_resource = !report.resources.iter().any(|value| value.id == resource.id);
    if new_resource {
        let bytes = report
            .resources
            .iter()
            .map(|value| value.byte_size)
            .sum::<u64>()
            + resource.byte_size;
        if bytes > super::model::MAX_TOTAL_RESOURCE_BYTES {
            return Err("Document visual resources exceed 256 MiB".into());
        }
    }
    report.occurrences[index].resource_id = Some(resource.id.clone());
    if new_resource {
        report.resources.push(resource);
    }
    Ok(())
}
