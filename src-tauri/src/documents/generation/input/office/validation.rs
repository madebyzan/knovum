use office_oxide::ir::{DocumentIR, Element, InlineContent, List};

use super::super::DocumentFormat;
use crate::documents::extraction::office::content;

#[derive(Default)]
struct ContentBudget {
    text_bytes: usize,
    element_count: usize,
    image_bytes: usize,
}

pub(crate) fn validate(ir: &DocumentIR, format: DocumentFormat) -> Result<(), String> {
    let mut budget = ContentBudget::default();
    if ir.sections.is_empty() || ir.sections.len() > 2_000 {
        return Err("Office conversion requires between 1 and 2000 sections".into());
    }
    for section in &ir.sections {
        if section.hidden {
            return Err("Hidden Office sections must be made visible before conversion".into());
        }
        if let Some(title) = &section.title {
            budget.validate_text(title)?;
        }
        for part in content::parts(section) {
            if matches!(format, DocumentFormat::Pptx) && part.part != "body" {
                return Err(
                    "Office slide conversion does not support page headers or footers".into(),
                );
            }
            budget.validate_elements(part.elements, format, 0)?;
        }
        if let Some(notes) = &section.speaker_notes {
            if !notes.is_empty() && matches!(format, DocumentFormat::Docx) {
                return Err("Word conversion does not support presentation speaker notes".into());
            }
            budget.validate_elements(notes, format, 0)?;
        }
    }
    Ok(())
}

impl ContentBudget {
    fn validate_text(&mut self, value: &str) -> Result<(), String> {
        self.text_bytes = self.text_bytes.saturating_add(value.len());
        if self.text_bytes > super::super::super::markdown::MAX_SOURCE_BYTES {
            return Err("Office conversion text exceeds 4 MiB".into());
        }
        super::super::super::markdown::validate_text(value)
    }

    fn validate_elements(
        &mut self,
        elements: &[Element],
        format: DocumentFormat,
        depth: usize,
    ) -> Result<(), String> {
        if depth > super::super::super::markdown::MAX_DEPTH {
            return Err("Office conversion nesting exceeds 64 levels".into());
        }
        self.element_count = self.element_count.saturating_add(elements.len());
        if self.element_count > super::super::super::markdown::MAX_EVENTS {
            return Err("Office conversion exceeds the content element budget".into());
        }
        for element in elements {
            match element {
                Element::Heading(value) => self.validate_inline(&value.content)?,
                Element::Paragraph(value) => self.validate_inline(&value.content)?,
                Element::CodeBlock(value) => self.validate_text(&value.content)?,
                Element::TextBox(value) => {
                    self.validate_elements(&value.content, format, depth + 1)?
                }
                Element::Footnote(value) | Element::Endnote(value) => {
                    self.validate_elements(&value.content, format, depth + 1)?
                }
                Element::List(value) => self.validate_list(value, format, depth + 1)?,
                Element::Table(value) => {
                    for row in &value.rows {
                        if row.cells.len() > 64 {
                            return Err("Office conversion tables exceed 64 columns".into());
                        }
                        for cell in &row.cells {
                            self.validate_elements(&cell.content, format, depth + 1)?;
                        }
                    }
                }
                Element::Image(value) => self.validate_image(value, format)?,
                Element::Shape(_) => {
                    return Err("Office conversion does not support vector drawings".into())
                }
                Element::ThematicBreak | Element::PageBreak | Element::ColumnBreak => {}
                _ => return Err("Office conversion encountered unsupported content".into()),
            }
        }
        Ok(())
    }

    fn validate_list(
        &mut self,
        list: &List,
        format: DocumentFormat,
        depth: usize,
    ) -> Result<(), String> {
        if depth > super::super::super::markdown::MAX_DEPTH {
            return Err("Office conversion nesting exceeds 64 levels".into());
        }
        for item in &list.items {
            self.validate_elements(&item.content, format, depth)?;
            if let Some(nested) = &item.nested {
                self.validate_list(nested, format, depth + 1)?;
            }
        }
        Ok(())
    }

    fn validate_inline(&mut self, content: &[InlineContent]) -> Result<(), String> {
        for inline in content {
            match inline {
                InlineContent::Text(span) => {
                    self.validate_text(&span.text)?;
                    if let Some(link) = &span.hyperlink {
                        super::super::super::links::validate(link)?;
                    }
                }
                InlineContent::LineBreak
                | InlineContent::FootnoteRef(_)
                | InlineContent::EndnoteRef(_)
                | InlineContent::CommentStart(_)
                | InlineContent::CommentRef(_) => {}
                _ => return Err("Office conversion encountered unsupported inline content".into()),
            }
        }
        Ok(())
    }

    fn validate_image(
        &mut self,
        image: &office_oxide::ir::Image,
        format: DocumentFormat,
    ) -> Result<(), String> {
        if let Some(alt) = &image.alt_text {
            self.validate_text(alt)?;
        }
        if let Some(link) = &image.hyperlink {
            super::super::super::links::validate(link)?;
        }
        if matches!(
            format,
            DocumentFormat::Docx
                | DocumentFormat::Pptx
                | DocumentFormat::Html
                | DocumentFormat::Pdf
        ) && (image.data.as_ref().is_none_or(Vec::is_empty) || image.format.is_none())
        {
            return Err(
                "Office conversion requires embedded image bytes and a supported image format"
                    .into(),
            );
        }
        if matches!(format, DocumentFormat::Html)
            && !matches!(
                image.format,
                Some(
                    office_oxide::ir::ImageFormat::Png
                        | office_oxide::ir::ImageFormat::Jpeg
                        | office_oxide::ir::ImageFormat::Gif
                        | office_oxide::ir::ImageFormat::Bmp
                )
            )
        {
            return Err(
                "Office HTML conversion does not support this embedded image format".into(),
            );
        }
        self.image_bytes = self
            .image_bytes
            .saturating_add(image.data.as_ref().map_or(0, Vec::len));
        if self.image_bytes > 64 * 1024 * 1024 {
            return Err("Office conversion images exceed 64 MiB".into());
        }
        Ok(())
    }
}
