use std::collections::{BTreeMap, BTreeSet};

use office_oxide::ir::{Element, Image, List};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::documents::extraction::visual::{
    has_meaningful_raster_dimensions, is_meaningful_emu_size, is_meaningful_pixel_size,
    is_repeated_watermark_or_template, DocumentImage,
};
use crate::spaces::knowledge::model::document_coverage::{
    MAX_COVERED_VISUALS, MAX_RESOURCE_BYTES, MAX_TOTAL_RESOURCE_BYTES,
};

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
        filter_repeated_visuals(self.entries)
    }

    pub(super) fn add(
        &mut self,
        image_id: String,
        location: Value,
        context: &str,
        alt_text: Option<&str>,
        bytes: Option<&[u8]>,
    ) -> Result<(), String> {
        if alt_text.is_some_and(|value| value.encode_utf16().count() > 16_000) {
            return Err("Office image alternate text exceeds the processing budget".into());
        }
        if bytes.is_some_and(|data| !has_meaningful_raster_dimensions(data)) {
            return Ok(());
        }
        if !self.reserve(bytes) {
            return Ok(());
        }
        self.entries.push(DocumentImage {
            id: image_id,
            location,
            context: context.into(),
            alt_text: alt_text.map(str::to_owned),
            data: bytes.map(<[u8]>::to_vec),
        });
        Ok(())
    }

    fn reserve(&mut self, bytes: Option<&[u8]>) -> bool {
        let size = bytes.map_or(0, <[u8]>::len);
        if size > MAX_RESOURCE_BYTES as usize
            || self.byte_size + size > MAX_TOTAL_RESOURCE_BYTES as usize
            || self.entries.len() >= MAX_COVERED_VISUALS
        {
            return false;
        }
        self.byte_size += size;
        true
    }
}

fn is_qualifying_ir_image(image: &Image) -> bool {
    if image.decorative {
        return false;
    }
    if let (Some(width), Some(height)) = (image.display_width_emu, image.display_height_emu) {
        if width > 0 && height > 0 && !is_meaningful_emu_size(width, height) {
            return false;
        }
    }
    if let (Some(width), Some(height)) = (image.pixel_width, image.pixel_height) {
        if width > 0 && height > 0 && !is_meaningful_pixel_size(width, height) {
            return false;
        }
    }
    true
}

fn filter_repeated_visuals(entries: Vec<DocumentImage>) -> Vec<DocumentImage> {
    let digests = entries
        .iter()
        .map(|entry| {
            entry.data.as_deref().map(|bytes| {
                let digest: [u8; 32] = Sha256::digest(bytes).into();
                digest
            })
        })
        .collect::<Vec<_>>();
    let mut containers_by_digest = BTreeMap::<[u8; 32], BTreeSet<String>>::new();
    let mut count_by_digest_container = BTreeMap::<([u8; 32], String), usize>::new();
    for (entry, digest) in entries.iter().zip(&digests) {
        let Some(digest) = *digest else {
            continue;
        };
        let container = container_key(&entry.location);
        if container != "legacy" {
            containers_by_digest
                .entry(digest)
                .or_default()
                .insert(container.clone());
        }
        *count_by_digest_container
            .entry((digest, container))
            .or_default() += 1;
    }
    let mut max_by_digest = BTreeMap::<[u8; 32], usize>::new();
    for ((digest, _), count) in count_by_digest_container {
        let max = max_by_digest.entry(digest).or_default();
        *max = (*max).max(count);
    }
    entries
        .into_iter()
        .zip(digests)
        .filter_map(|(entry, digest)| {
            let repeated = digest.is_some_and(|key| {
                let containers = containers_by_digest.get(&key).map_or(0, BTreeSet::len);
                let max_in_one = max_by_digest.get(&key).copied().unwrap_or(0);
                is_repeated_watermark_or_template(containers, max_in_one)
            });
            (!repeated).then_some(entry)
        })
        .collect()
}

fn container_key(location: &Value) -> String {
    if let Some(section) = location.get("section").and_then(Value::as_u64) {
        return format!("section:{section}");
    }
    if let Some(slide) = location.get("slide").and_then(Value::as_u64) {
        return format!("slide:{slide}");
    }
    if let Some(sheet) = location.get("sheet").and_then(Value::as_str) {
        return format!("sheet:{sheet}");
    }
    "legacy".into()
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
                if is_qualifying_ir_image(image) {
                    output.add(
                        id,
                        location.clone(),
                        context,
                        image.alt_text.as_deref(),
                        image.data.as_deref(),
                    )?;
                }
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

#[cfg(test)]
#[path = "tests/images.rs"]
mod tests;
