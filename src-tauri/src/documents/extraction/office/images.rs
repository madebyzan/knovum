use office_oxide::ir::{Element, List};
use serde_json::Value;

use crate::documents::extraction::visual::DocumentImage;

#[derive(Default)]
pub(super) struct DocumentImages {
    entries: Vec<DocumentImage>,
    byte_size: usize,
}

impl DocumentImages {
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn attach_context(&mut self, start: usize, context: &str) {
        for image in &mut self.entries[start..] {
            image.context = context.to_owned();
        }
    }

    pub(super) fn into_entries(self) -> Vec<DocumentImage> {
        self.entries
    }

    pub(super) fn add(
        &mut self,
        id: String,
        location: Value,
        context: &str,
        alt_text: Option<&str>,
        bytes: Option<&[u8]>,
    ) -> Result<(), String> {
        if alt_text.is_some_and(|value| value.encode_utf16().count() > 16_000) {
            return Err("Office image alternate text exceeds the processing budget".into());
        }
        self.reserve(bytes)?;
        self.entries.push(DocumentImage {
            id,
            location,
            context: context.into(),
            alt_text: alt_text.map(str::to_owned),
            data: bytes.map(<[u8]>::to_vec),
        });
        Ok(())
    }

    fn reserve(&mut self, bytes: Option<&[u8]>) -> Result<(), String> {
        let size = bytes.map_or(0, <[u8]>::len);
        if size > 32 * 1024 * 1024 || self.byte_size + size > 256 * 1024 * 1024 {
            return Err("Office embedded image bytes exceed the processing budget".into());
        }
        if self.entries.len() >= 2000 {
            return Err("Office image count exceeds 2000".into());
        }
        self.byte_size += size;
        Ok(())
    }
}

pub(super) fn collect_legacy(
    document: &office_oxide::Document,
    output: &mut DocumentImages,
) -> Result<(), String> {
    let images = if let Some(doc) = document.as_doc() {
        doc.images()
    } else if let Some(doc) = document.as_xls() {
        doc.images()
    } else if let Some(doc) = document.as_ppt() {
        doc.images()
    } else {
        return Ok(());
    };
    for image in images {
        output.add(
            format!("legacy:stream-image:{}", image.index),
            serde_json::json!({"kind":"node"}),
            "",
            None,
            Some(&image.data),
        )?;
    }
    Ok(())
}

pub(super) fn collect(
    elements: &[Element],
    location: &Value,
    context: &str,
    prefix: &str,
    output: &mut DocumentImages,
    depth: usize,
) -> Result<(), String> {
    if depth > 64 {
        return Err("Office nesting exceeds the processing budget".into());
    }
    for (index, element) in elements.iter().enumerate() {
        let id = format!("{prefix}/{index}");
        match element {
            Element::Image(image) => {
                output.add(
                    id,
                    location.clone(),
                    context,
                    image.alt_text.as_deref(),
                    image.data.as_deref(),
                )?;
            }
            Element::TextBox(box_) => {
                let mut owner = location.clone();
                if owner["kind"] == "sheet" {
                    if let (Some(x), Some(y), Some(width), Some(height)) =
                        (box_.x_emu, box_.y_emu, box_.width_emu, box_.height_emu)
                    {
                        if width > 0 && height > 0 {
                            owner["drawing"] = serde_json::json!({"xEmu":x,"yEmu":y,"widthEmu":width,"heightEmu":height});
                        }
                    }
                }
                collect(&box_.content, &owner, context, &id, output, depth + 1)?
            }
            Element::Footnote(note) | Element::Endnote(note) => {
                collect(&note.content, location, context, &id, output, depth + 1)?
            }
            Element::Table(table) => {
                for (row_index, row) in table.rows.iter().enumerate() {
                    for (cell_index, cell) in row.cells.iter().enumerate() {
                        collect(
                            &cell.content,
                            location,
                            context,
                            &format!("{id}/r{row_index}/c{cell_index}"),
                            output,
                            depth + 1,
                        )?;
                    }
                }
            }
            Element::List(list) => collect_list(list, location, context, &id, output, depth + 1)?,
            Element::Heading(_)
            | Element::Paragraph(_)
            | Element::CodeBlock(_)
            | Element::ThematicBreak
            | Element::PageBreak
            | Element::ColumnBreak
            | Element::Shape(_) => {}
            _ => return Err("Unsupported Office content element".into()),
        }
    }
    Ok(())
}

fn collect_list(
    list: &List,
    location: &Value,
    context: &str,
    prefix: &str,
    output: &mut DocumentImages,
    depth: usize,
) -> Result<(), String> {
    if depth > 64 {
        return Err("Office list nesting exceeds the processing budget".into());
    }
    for (index, entry) in list.items.iter().enumerate() {
        let prefix = format!("{prefix}/i{index}");
        collect(
            &entry.content,
            location,
            context,
            &prefix,
            output,
            depth + 1,
        )?;
        if let Some(nested) = &entry.nested {
            collect_list(nested, location, context, &prefix, output, depth + 1)?;
        }
    }
    Ok(())
}
