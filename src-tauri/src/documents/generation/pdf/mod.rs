mod rich;
mod rich_blocks;
mod rich_inlines;
mod rich_tables;
mod source;

pub(super) fn render_ir(ir: &office_oxide::ir::DocumentIR) -> Result<Vec<u8>, String> {
    rich::render(ir)
}

use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};
use typst_layout::PagedDocument;

use super::document::DocumentSource;

pub(crate) fn render(input: &DocumentSource) -> Result<Vec<u8>, String> {
    let document = super::markdown::parse(input)?;
    let images = super::images::Images::load(input)?;
    let source = source::render(&document, &images)?;
    compile(source, images.binaries())
}

pub(super) fn compile(source: String, binaries: Vec<(String, Vec<u8>)>) -> Result<Vec<u8>, String> {
    let engine = TypstEngine::builder()
        .main_file(source)
        .with_static_file_resolver(
            binaries
                .iter()
                .map(|(name, bytes)| (name.as_str(), bytes.as_slice())),
        )
        .search_fonts_with(TypstKitFontOptions::default())
        .build();
    let compilation = engine.compile();
    if !compilation.warnings.is_empty() {
        log::warn!(
            "Document typesetting produced {} warnings",
            compilation.warnings.len()
        );
    }
    let document: PagedDocument = compilation
        .output
        .map_err(|errors| format!("PDF typesetting failed: {errors:?}"))?;
    typst_pdf::pdf(&document, &Default::default())
        .map_err(|errors| format!("PDF export failed: {errors:?}"))
}

#[cfg(test)]
pub(crate) mod tests;
