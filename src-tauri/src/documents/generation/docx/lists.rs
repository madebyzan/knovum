use docx_rs::{
    AbstractNumbering, IndentLevel, Level, LevelJc, LevelText, NumberFormat, Numbering,
    NumberingId, Paragraph, SpecialIndentType, Start,
};

use super::super::document::Block;
use super::{inlines, Writer};

impl Writer {
    pub(super) fn list(
        &mut self,
        start: Option<u64>,
        items: &[Vec<Block>],
        indent: i32,
    ) -> Result<(), String> {
        let id = self.next_numbering;
        self.next_numbering += 1;
        let level = Level::new(
            0,
            Start::new(start.unwrap_or(1) as usize),
            NumberFormat::new(if start.is_some() { "decimal" } else { "bullet" }),
            LevelText::new(if start.is_some() { "%1." } else { "•" }),
            LevelJc::new("left"),
        )
        .indent(
            Some(indent + 360),
            Some(SpecialIndentType::Hanging(240)),
            None,
            None,
        );
        self.document = std::mem::take(&mut self.document)
            .add_abstract_numbering(AbstractNumbering::new(id).add_level(level))
            .add_numbering(Numbering::new(id, id));
        for item in items {
            let (first, rest) = match item.first() {
                Some(Block::Paragraph(content)) => (inlines::paragraph(content)?, &item[1..]),
                _ => (Paragraph::new(), item.as_slice()),
            };
            self.paragraph(
                first
                    .style("Normal")
                    .numbering(NumberingId::new(id), IndentLevel::new(0)),
            );
            self.blocks(rest, indent + 360)?;
        }
        Ok(())
    }
}
