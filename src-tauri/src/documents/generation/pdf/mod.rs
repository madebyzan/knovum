mod source;

use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};
use typst_layout::PagedDocument;

use super::document::DocumentSource;

pub(crate) fn render(input: &DocumentSource) -> Result<Vec<u8>, String> {
    let document = super::markdown::parse(input)?;
    let source = source::render(&document)?;
    let engine = TypstEngine::builder()
        .main_file(source)
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
pub(crate) mod test_support;
