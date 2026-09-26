use docx_rs::{AlignmentType, Table, TableCell, TableRow};

use super::super::document::{Alignment, Table as DocumentTable};
use super::inlines;

pub(super) fn render(table: &DocumentTable) -> Result<Table, String> {
    let mut rows = Vec::new();
    for (index, row) in std::iter::once(&table.header)
        .chain(&table.rows)
        .enumerate()
    {
        let mut cells = Vec::new();
        for (content, alignment) in row.iter().zip(&table.alignments) {
            let paragraph = if index == 0 {
                inlines::header_paragraph(content)?.keep_next(true)
            } else {
                inlines::paragraph(content)?
            };
            let paragraph = paragraph.style("KnovumBody").align(match alignment {
                Alignment::Left => AlignmentType::Left,
                Alignment::Center => AlignmentType::Center,
                Alignment::Right => AlignmentType::Right,
            });
            cells.push(TableCell::new().add_paragraph(paragraph));
        }
        rows.push(TableRow::new(cells));
    }
    Ok(Table::new(rows))
}
