use office_oxide::xlsx::{worksheet::Row, XlsxDocument};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::DocumentSegment;

/// Public cell APIs preserve actual addresses, formulas and cached values without IR's row cap.
pub(super) fn segments(
    document: &XlsxDocument,
    cancellation: &CancellationToken,
    budget: &mut super::budget::TextBudget,
) -> Result<Vec<DocumentSegment>, String> {
    let mut segments = Vec::new();
    for sheet in &document.worksheets {
        let header = sheet
            .rows
            .first()
            .map(|row| super::super::text::bounded_utf16_text(&row_text(document, row), 4000))
            .unwrap_or_default();
        for group in sheet.rows.chunks(24) {
            super::ensure_active(cancellation)?;
            let Some(first) = group.first() else { continue };
            let last = group.last().ok_or("Worksheet row group is empty")?;
            let body = group
                .iter()
                .map(|row| row_text(document, row))
                .collect::<Vec<_>>()
                .join("\n");
            if body.trim().is_empty() {
                continue;
            }
            let text = if first.index == sheet.rows[0].index {
                body
            } else {
                format!("First row context (not a declared header):\n{header}\n\n{body}")
            };
            budget.consume(&text)?;
            segments.push(DocumentSegment { kind: "table", text,
                location: json!({"kind":"sheet", "sheet":sheet.name, "range":format!("{}:{}", first.index, last.index)}),
                metadata: json!({"rowStart":first.index,"rowEnd":last.index,"formulas":"stored_formula_and_cached_value"}) });
        }
        for (index, shape) in sheet.text_shapes.iter().enumerate() {
            super::ensure_active(cancellation)?;
            if shape.text.trim().is_empty() {
                continue;
            }
            budget.consume(&shape.text)?;
            segments.push(DocumentSegment {
                kind: "text",
                text: shape.text.clone(),
                location: super::location::sheet_drawing(
                    &sheet.name,
                    index,
                    shape.x_emu,
                    shape.y_emu,
                    shape.cx_emu,
                    shape.cy_emu,
                ),
                metadata: json!({"role":"worksheet_text_shape","shapeIndex":index}),
            });
        }
    }
    for (index, text) in document.chart_text.iter().enumerate() {
        if !text.trim().is_empty() {
            budget.consume(text)?;
            segments.push(DocumentSegment { kind: "text", text: text.clone(), location: json!({"kind":"node"}),
                metadata: json!({"role":"chart_text","chartIndex":index,"position":"workbook_order_not_sheet_anchor"}) });
        }
    }
    Ok(segments)
}

fn row_text(document: &XlsxDocument, row: &Row) -> String {
    row.cells
        .iter()
        .map(|cell| {
            let address = cell.reference.to_string();
            let value = document.format_cell_value(cell);
            match &cell.formula {
                Some(formula) => format!("{address}: formula={formula}; cached={value}"),
                None => format!("{address}: {value}"),
            }
        })
        .collect::<Vec<_>>()
        .join("\t")
}

pub(super) fn legacy_segments(
    document: &office_oxide::xls::XlsDocument,
    cancellation: &CancellationToken,
    budget: &mut super::budget::TextBudget,
) -> Result<Vec<DocumentSegment>, String> {
    let mut output = Vec::new();
    for sheet in &document.sheets {
        for (group_index, rows) in sheet.rows.chunks(24).enumerate() {
            super::ensure_active(cancellation)?;
            let first = group_index * 24 + 1;
            let text = rows
                .iter()
                .enumerate()
                .map(|(row_index, row)| {
                    row.iter()
                        .enumerate()
                        .map(|(column, cell)| {
                            format!("R{}C{}: {}", first + row_index, column + 1, cell.as_text())
                        })
                        .collect::<Vec<_>>()
                        .join("\t")
                })
                .collect::<Vec<_>>()
                .join("\n");
            if text.trim().is_empty() {
                continue;
            }
            budget.consume(&text)?;
            output.push(DocumentSegment { kind:"table",text,
                location:json!({"kind":"sheet","sheet":sheet.name,"range":format!("{}:{}",first,first+rows.len()-1)}),
                metadata:json!({"rowStart":first,"rowEnd":first+rows.len()-1,"formulas":"cached_values_only"}) });
        }
    }
    Ok(output)
}
