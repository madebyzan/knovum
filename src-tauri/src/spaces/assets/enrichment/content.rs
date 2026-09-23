use std::path::Path;

use tokio_util::sync::CancellationToken;

use super::{ensure_active, text_chunks};
use crate::{
    documents::extraction::{office, pdf, DocumentSegment},
    spaces::{
        assets::PreparedAsset,
        knowledge::indexing::{chunks, KnowledgeChunkSeed},
    },
};

/// Publishes parsed source text separately from the canvas's visual thumbnail.
pub(in crate::spaces::assets) fn extract(
    app: &tauri::AppHandle,
    source: &Path,
    revision: &str,
    asset: &mut PreparedAsset,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    ensure_active(cancellation)?;
    let pdf_source = mime_guess::from_path(source).first_raw() == Some("application/pdf");
    let document_source = pdf_source || office::supports(source);
    if document_source && crate::file_inspection::content_revision(source)? != revision {
        return Err("Asset source changed before content extraction".into());
    }
    if asset.preview_language.as_deref() == Some("html") {
        asset.preview_path = Some(source.to_string_lossy().into_owned());
        text_chunks::attach(asset, revision);
        return Ok(());
    }
    if pdf_source {
        let document = pdf::extract(source, cancellation)?;
        let coverage =
            super::document::prepare::pdf(app, source, &document, revision, cancellation)?;
        asset.preview_text = document.preview_text;
        asset.extracted_text = document.extracted_text;
        asset.knowledge_chunks = Some(source_chunks(document.segments, revision));
        asset.document_coverage =
            Some(serde_json::to_value(coverage).map_err(|error| error.to_string())?);
    } else if office::supports(source) {
        let document = office::extract(source, cancellation)?;
        let coverage = super::document::prepare::office(
            app,
            document.images,
            document.warnings,
            revision,
            cancellation,
        )?;
        asset.preview_text = document.preview_text;
        asset.extracted_text = document.extracted_text;
        asset.knowledge_chunks = Some(source_chunks(document.segments, revision));
        asset.document_coverage =
            Some(serde_json::to_value(coverage).map_err(|error| error.to_string())?);
    } else if crate::file_inspection::classify_file(source)
        == crate::file_inspection::ResourceClassification::Archive
    {
        if asset.entries.is_none() {
            let entries = crate::file_inspection::archive_entries(source, &asset.name)?;
            asset.preview_text = Some(crate::file_inspection::entry_search_text(&entries));
            asset.entries = Some(entries);
        }
    } else {
        text_chunks::attach(asset, revision);
    }
    ensure_active(cancellation)?;
    if document_source && crate::file_inspection::content_revision(source)? != revision {
        return Err("Asset source changed during content extraction".into());
    }
    Ok(())
}

fn source_chunks(segments: Vec<DocumentSegment>, revision: &str) -> Vec<KnowledgeChunkSeed> {
    segments
        .into_iter()
        .flat_map(|segment| {
            chunks(
                segment.kind,
                &segment.text,
                revision,
                segment.location,
                segment.metadata,
            )
        })
        .collect()
}
