use office_oxide::ir::{Element, InlineContent, List};

pub(super) const MAX_TEXT_UTF16: usize = 4 * 1024 * 1024;

#[derive(Default)]
struct Text {
    value: String,
    length: usize,
}

impl Text {
    fn append(&mut self, value: &str) -> Result<(), String> {
        let length = value.encode_utf16().count();
        if self.length + length > MAX_TEXT_UTF16 {
            return Err("Office text exceeds the processing budget".into());
        }
        self.value.push_str(value);
        self.length += length;
        Ok(())
    }

    fn line(&mut self) -> Result<(), String> {
        if !self.value.is_empty() && !self.value.ends_with('\n') {
            self.append("\n")?;
        }
        Ok(())
    }
}

pub(super) fn table_row_text(row: &office_oxide::ir::TableRow) -> Result<String, String> {
    let mut text = Text::default();
    for (index, cell) in row.cells.iter().enumerate() {
        if index > 0 {
            text.append("\t")?;
        }
        append_elements(&mut text, &cell.content, 0)?;
    }
    Ok(text.value.trim().into())
}

pub(super) fn element_text(element: &Element) -> Result<String, String> {
    let mut text = Text::default();
    append_element(&mut text, element, 0)?;
    Ok(text.value.trim().into())
}

fn append_element(text: &mut Text, element: &Element, depth: usize) -> Result<(), String> {
    ensure_depth(depth)?;
    match element {
        Element::Heading(value) => append_inline(text, &value.content)?,
        Element::Paragraph(value) => append_inline(text, &value.content)?,
        Element::Table(value) => {
            for row in &value.rows {
                text.line()?;
                for (index, cell) in row.cells.iter().enumerate() {
                    if index > 0 {
                        text.append("\t")?;
                    }
                    append_elements(text, &cell.content, depth + 1)?;
                }
            }
        }
        Element::List(value) => append_list(text, value, depth + 1)?,
        Element::Image(value) => text.append(value.alt_text.as_deref().unwrap_or(""))?,
        Element::TextBox(value) => append_elements(text, &value.content, depth + 1)?,
        Element::Footnote(value) | Element::Endnote(value) => {
            append_elements(text, &value.content, depth + 1)?;
        }
        Element::CodeBlock(value) => text.append(&value.content)?,
        Element::ThematicBreak | Element::PageBreak | Element::ColumnBreak | Element::Shape(_) => {}
        _ => return Err("Unsupported Office content element".into()),
    }
    Ok(())
}

fn append_list(text: &mut Text, list: &List, depth: usize) -> Result<(), String> {
    ensure_depth(depth)?;
    for (index, entry) in list.items.iter().enumerate() {
        text.line()?;
        if list.ordered {
            text.append(&format!(
                "{}. ",
                u64::from(list.start_number.unwrap_or(1)) + index as u64
            ))?;
        } else {
            text.append("• ")?;
        }
        append_elements(text, &entry.content, depth + 1)?;
        if let Some(nested) = &entry.nested {
            append_list(text, nested, depth + 1)?;
        }
    }
    Ok(())
}

fn append_elements(text: &mut Text, elements: &[Element], depth: usize) -> Result<(), String> {
    ensure_depth(depth)?;
    for (index, element) in elements.iter().enumerate() {
        if index > 0 {
            text.line()?;
        }
        append_element(text, element, depth)?;
    }
    Ok(())
}

fn append_inline(text: &mut Text, content: &[InlineContent]) -> Result<(), String> {
    for inline in content {
        match inline {
            InlineContent::Text(span) => text.append(&span.text)?,
            InlineContent::LineBreak => text.append("\n")?,
            InlineContent::FootnoteRef(reference) | InlineContent::EndnoteRef(reference) => {
                match &reference.marker {
                    Some(marker) => text.append(marker)?,
                    None => text.append(&reference.note_id.to_string())?,
                }
            }
            _ => return Err("Unsupported Office inline content".into()),
        }
    }
    Ok(())
}

fn ensure_depth(depth: usize) -> Result<(), String> {
    if depth > 64 {
        Err("Office nesting exceeds the processing budget".into())
    } else {
        Ok(())
    }
}
