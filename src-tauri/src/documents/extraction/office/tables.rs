use office_oxide::ir::Table;
use serde_json::{json, Value};
use tokio_util::sync::CancellationToken;

use super::{document_text::table_row_text, DocumentSegment};

pub(super) fn segments(
    table: &Table,
    location: Value,
    cancellation: &CancellationToken,
    budget: &mut super::budget::TextBudget,
) -> Result<Vec<DocumentSegment>, String> {
    let header = table
        .rows
        .first()
        .filter(|row| row.is_header)
        .map(table_row_text)
        .transpose()?
        .map(|text| super::super::text::bounded_utf16_text(&text, 4000));
    let mut output = Vec::new();
    for (index, rows) in table.rows.chunks(24).enumerate() {
        super::ensure_active(cancellation)?;
        let body = rows
            .iter()
            .map(table_row_text)
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");
        if body.trim().is_empty() {
            continue;
        }
        let text = match (&header, index) {
            (Some(header), index) if index > 0 => format!("Table header:\n{header}\n\n{body}"),
            _ => body,
        };
        budget.consume(&text)?;
        output.push(DocumentSegment {
            kind: "table",
            text,
            location: location.clone(),
            metadata: json!({"tableRowStart":index*24+1,"tableRowEnd":index*24+rows.len()}),
        });
    }
    Ok(output)
}
