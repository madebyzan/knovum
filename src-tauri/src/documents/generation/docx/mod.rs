mod inlines;
mod lists;
mod styles;
mod tables;

use std::io::Cursor;

use docx_rs::{Docx, Paragraph, Run};

use super::document::{Block, Document, DocumentSource};

pub(crate) fn render(source: &DocumentSource) -> Result<Vec<u8>, String> {
    let document = super::markdown::parse(source)?;
    let mut cursor = Cursor::new(Vec::new());
    build_document(&document)?
        .build()
        .pack(&mut cursor)
        .map_err(|error| error.to_string())?;
    Ok(cursor.into_inner())
}

pub(super) fn build_document(document: &Document) -> Result<Docx, String> {
    let mut writer = Writer {
        document: styles::document(),
        next_numbering: 1,
    };
    if !document.title.is_empty() {
        writer.paragraph(
            Paragraph::new()
                .style("Title")
                .keep_next(true)
                .add_run(inlines::text_run(&document.title)),
        );
    }
    writer.blocks(&document.blocks, 0)?;
    Ok(writer.document)
}

struct Writer {
    document: Docx,
    next_numbering: usize,
}

impl Writer {
    fn blocks(&mut self, blocks: &[Block], indent: i32) -> Result<(), String> {
        for block in blocks {
            match block {
                Block::Paragraph(content) => {
                    let paragraph = inlines::paragraph(content)?.style("KnovumBody").indent(
                        Some(indent),
                        None,
                        None,
                        None,
                    );
                    self.paragraph(paragraph);
                }
                Block::Heading { level, content } => {
                    let paragraph = inlines::paragraph(content)?
                        .style(&format!("Heading{level}"))
                        .outline_lvl(usize::from(*level - 1))
                        .keep_next(true)
                        .indent(Some(indent), None, None, None);
                    self.paragraph(paragraph);
                }
                Block::Code { text, .. } => {
                    let paragraph = inlines::code_paragraph(text)
                        .style("KnovumCodeBlock")
                        .indent(Some(indent), None, None, None);
                    self.paragraph(paragraph);
                }
                Block::Quote(children) => self.blocks(children, indent + 360)?,
                Block::List { start, items } => self.list(*start, items, indent)?,
                Block::Table(value) => {
                    let table = tables::render(value)?;
                    self.document = std::mem::take(&mut self.document).add_table(table);
                }
                Block::Rule => {
                    self.paragraph(Paragraph::new().add_run(Run::new().add_text("────────")));
                }
            }
        }
        Ok(())
    }

    fn paragraph(&mut self, paragraph: Paragraph) {
        self.document = std::mem::take(&mut self.document).add_paragraph(paragraph);
    }
}
