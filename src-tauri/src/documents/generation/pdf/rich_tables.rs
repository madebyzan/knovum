use office_oxide::ir::{ParagraphAlignment, Table};

use super::rich::Writer;

impl Writer {
    pub(super) fn table(&mut self, table: &Table) -> Result<(), String> {
        let columns = table
            .rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| cell.col_span.max(1))
                    .sum::<u32>()
            })
            .max()
            .unwrap_or(0);
        if columns == 0 || columns > 64 {
            return Err("Document tables require between 1 and 64 columns".into());
        }
        self.source
            .push_str(&format!("#table(columns: {columns},\n"));
        for row in &table.rows {
            if row.is_header {
                self.source.push_str("table.header(");
            }
            for cell in &row.cells {
                let alignment = match cell.text_align {
                    Some(ParagraphAlignment::Center) => "center",
                    Some(ParagraphAlignment::Right) => "right",
                    _ => "left",
                };
                self.source.push_str(&format!(
                    "table.cell(colspan: {}, rowspan: {}, align: {alignment},",
                    cell.col_span.max(1),
                    cell.row_span.max(1)
                ));
                if let Some(rgb) = cell.background_color {
                    self.source
                        .push_str(&format!("fill: rgb({}, {}, {}),", rgb[0], rgb[1], rgb[2]));
                }
                self.source.push_str("[\n");
                self.blocks(&cell.content)?;
                self.source.push_str("]),\n");
            }
            if row.is_header {
                self.source.push_str("),\n");
            }
        }
        self.source.push(')');
        Ok(())
    }
}
