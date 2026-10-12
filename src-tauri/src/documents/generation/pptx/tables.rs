use super::{
    pagination::{Paginator, BODY_LINES, LINE_WIDTH},
    paragraphs::{self, Chunk},
};
use office_oxide::ir::{Element, InlineContent, Paragraph, Table, TableCell, TableRow};

impl Paginator {
    pub fn table(&mut self, table: &Table) -> Result<(), String> {
        if self
            .current
            .elements
            .iter()
            .any(|element| !matches!(element, Element::Heading(_)))
        {
            self.flush(false)?;
        } else {
            self.current.elements.clear();
        }
        let columns = table
            .rows
            .iter()
            .map(|row| row.cells.len())
            .max()
            .unwrap_or(0);
        if columns == 0 {
            return Ok(());
        }
        if columns > 12 {
            return Err("Presentation tables exceed the readable 12-column layout".into());
        }
        let mut headers: Vec<TableRow> = Vec::new();
        let mut header_lines = 0;
        let mut repeat_headers = true;
        let mut chunk = table.clone();
        chunk.rows.clear();
        let mut lines = 0;
        for row in &table.rows {
            let mut cells = Vec::new();
            for cell in &row.cells {
                // Leave room for the writer's default left/right cell padding.
                cells.push(split_cell(
                    cell,
                    (LINE_WIDTH / columns).saturating_sub(1).max(1),
                )?);
            }
            let pieces = cells.iter().map(Vec::len).max().unwrap_or(1);
            for piece in 0..pieces {
                let height = cells
                    .iter()
                    .filter_map(|cell| cell.get(piece))
                    .map(|value| value.lines)
                    .max()
                    .unwrap_or(1)
                    + 1;
                if lines + height > BODY_LINES && !chunk.rows.is_empty() {
                    self.current.elements.push(Element::Table(chunk.clone()));
                    self.flush(false)?;
                    chunk.rows =
                        if repeat_headers && pieces == 1 && header_lines + height <= BODY_LINES {
                            headers.clone()
                        } else {
                            vec![]
                        };
                    lines = if chunk.rows.is_empty() {
                        0
                    } else {
                        header_lines
                    };
                }
                let mut new_row = TableRow {
                    is_header: row.is_header,
                    ..Default::default()
                };
                for (cell, parts) in row.cells.iter().zip(&cells) {
                    let mut new_cell = cell.clone();
                    new_cell.content = vec![Element::Paragraph(Paragraph {
                        content: parts
                            .get(piece)
                            .map(|part| part.content.clone())
                            .unwrap_or_default(),
                        ..Default::default()
                    })];
                    new_row.cells.push(new_cell);
                }
                if row.is_header {
                    if pieces == 1 && header_lines + height < BODY_LINES {
                        headers.push(new_row.clone());
                        header_lines += height;
                    } else {
                        repeat_headers = false;
                        headers.clear();
                        header_lines = 0;
                    }
                }
                chunk.rows.push(new_row);
                lines += height;
            }
        }
        if !chunk.rows.is_empty() {
            self.current.elements.push(Element::Table(chunk));
            self.flush(false)?;
        }
        Ok(())
    }
}

fn split_cell(cell: &TableCell, width: usize) -> Result<Vec<Chunk>, String> {
    let mut content = Vec::new();
    for (index, element) in cell.content.iter().enumerate() {
        if index > 0 {
            content.push(InlineContent::LineBreak);
        }
        match element {
            Element::Paragraph(value) => content.extend(value.content.clone()),
            _ => return Err("Presentation table cells require text content".into()),
        }
    }
    // The admitted writer emits 36pt rows; one 20pt line fits without clipping.
    paragraphs::split(&content, width, 1)
}
