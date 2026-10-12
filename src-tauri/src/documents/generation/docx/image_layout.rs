use office_oxide::ir::{DocumentIR, Element, Image, List};
use std::io::Cursor;

const MAX_WIDTH_EMU: u64 = 546 * 9_525;
const MAX_HEIGHT_EMU: u64 = 850 * 9_525;

pub(super) fn pixel_size(width: u32, height: u32) -> (u64, u64) {
    fit(u64::from(width) * 9_525, u64::from(height) * 9_525)
}

pub(in crate::documents::generation) fn prepare(ir: &mut DocumentIR) -> Result<(), String> {
    for section in &mut ir.sections {
        elements(&mut section.elements)?;
    }
    Ok(())
}

fn fit(width: u64, height: u64) -> (u64, u64) {
    let scale = (MAX_WIDTH_EMU as f64 / width as f64)
        .min(MAX_HEIGHT_EMU as f64 / height as f64)
        .min(1.0);
    (
        (width as f64 * scale).round().max(1.0) as u64,
        (height as f64 * scale).round().max(1.0) as u64,
    )
}

fn elements(content: &mut [Element]) -> Result<(), String> {
    for element in content {
        match element {
            Element::Image(image) => {
                let (width, height) = dimensions(image)?;
                let (width, height) = fit(width, height);
                image.display_width_emu = Some(width);
                image.display_height_emu = Some(height);
            }
            Element::Table(table) => {
                for cell in table.rows.iter_mut().flat_map(|row| &mut row.cells) {
                    elements(&mut cell.content)?;
                }
            }
            Element::List(value) => list(value)?,
            Element::TextBox(value) => elements(&mut value.content)?,
            Element::Footnote(value) | Element::Endnote(value) => elements(&mut value.content)?,
            _ => {}
        }
    }
    Ok(())
}

fn list(value: &mut List) -> Result<(), String> {
    for item in &mut value.items {
        elements(&mut item.content)?;
        if let Some(nested) = &mut item.nested {
            list(nested)?;
        }
    }
    Ok(())
}

fn dimensions(image: &Image) -> Result<(u64, u64), String> {
    if let (Some(width), Some(height)) = (image.display_width_emu, image.display_height_emu) {
        if width == 0 || height == 0 {
            return Err("DOCX image dimensions must be positive".into());
        }
        return Ok((width, height));
    }
    let (width, height) =
        if let (Some(width), Some(height)) = (image.pixel_width, image.pixel_height) {
            (width, height)
        } else {
            let bytes = image.data.as_deref().ok_or("DOCX image data is missing")?;
            image::ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()
                .map_err(|error| error.to_string())?
                .into_dimensions()
                .map_err(|error| format!("DOCX image dimensions are unavailable: {error}"))?
        };
    if width == 0 || height == 0 {
        return Err("DOCX image dimensions must be positive".into());
    }
    Ok((u64::from(width) * 9_525, u64::from(height) * 9_525))
}
