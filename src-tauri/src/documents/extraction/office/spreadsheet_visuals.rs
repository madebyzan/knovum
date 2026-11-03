use super::{ensure_active, images::DocumentImages, location::sheet_drawing};
use crate::documents::extraction::visual::is_meaningful_emu_size;
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
            if image.cx_emu > 0
                && image.cy_emu > 0
                && !is_meaningful_emu_size(image.cx_emu as u64, image.cy_emu as u64)
            {
                continue;
            }
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
