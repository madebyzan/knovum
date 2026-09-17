use super::{ensure_active, images::DocumentImages, location::sheet_drawing};
use office_oxide::xlsx::XlsxDocument;
use tokio_util::sync::CancellationToken;

/// Public worksheet pictures avoid constructing and cloning the workbook's complete table IR.
pub(super) fn collect(
    document: &XlsxDocument,
    cancellation: &CancellationToken,
) -> Result<DocumentImages, String> {
    let mut images = DocumentImages::default();
    for (sheet_index, sheet) in document.worksheets.iter().enumerate() {
        for (index, image) in sheet.images.iter().enumerate() {
            ensure_active(cancellation)?;
            let location = sheet_drawing(
                &sheet.name,
                index,
                image.x_emu,
                image.y_emu,
                image.cx_emu,
                image.cy_emu,
            );
            let context = super::super::text::bounded_utf16_text(&sheet.name, 4000);
            images.add(
                format!("xlsx:{sheet_index}/image:{index}"),
                location,
                &context,
                image.alt_text.as_deref(),
                Some(&image.data),
            )?;
        }
    }
    Ok(images)
}
