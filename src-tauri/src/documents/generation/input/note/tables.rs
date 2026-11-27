use office_oxide::ir::{Element, Table, TableRow};

use super::{model, table_cells, Context};
use model::Node;

pub(super) fn parse(node: &Node, context: &Context<'_>, depth: usize) -> Result<Element, String> {
    model::no_attrs(&node.attrs, "table")?;
    if node.content.is_empty() {
        return Err("Tiptap tables require at least one row".into());
    }
    let mut occupancy = Vec::<u32>::new();
    let mut widths = Vec::<Option<u32>>::new();
    let mut rows = Vec::new();
    for node in &node.content {
        rows.push(row(node, context, depth + 1, &mut occupancy, &mut widths)?);
        occupancy
            .iter_mut()
            .for_each(|remaining| *remaining = remaining.saturating_sub(1));
    }
    let columns = occupancy.len().max(widths.len());
    if columns == 0 || columns > 64 {
        return Err("Tiptap tables require between 1 and 64 columns".into());
    }
    let column_widths_twips = resolve_column_widths(&widths);
    Ok(Element::Table(Table {
        rows,
        column_widths_twips,
        ..Default::default()
    }))
}

const DEFAULT_TABLE_WIDTH_TWIPS: u32 = 9_360;
const MIN_COLUMN_WIDTH_TWIPS: u32 = 1_080;

fn resolve_column_widths(widths: &[Option<u32>]) -> Vec<u32> {
    if widths.iter().all(Option::is_none) {
        return Vec::new();
    }
    if widths.iter().all(Option::is_some) {
        return widths.iter().map(|width| width.unwrap()).collect();
    }
    let specified_sum: u32 = widths
        .iter()
        .flatten()
        .fold(0u32, |acc, &width| acc.saturating_add(width));
    let unspecified = widths.iter().filter(|width| width.is_none()).count() as u32;
    let remaining = DEFAULT_TABLE_WIDTH_TWIPS.saturating_sub(specified_sum);
    let fallback = (remaining / unspecified.max(1)).max(MIN_COLUMN_WIDTH_TWIPS);
    widths
        .iter()
        .map(|width| width.unwrap_or(fallback))
        .collect()
}

fn row(
    node: &Node,
    context: &Context<'_>,
    depth: usize,
    occupancy: &mut Vec<u32>,
    widths: &mut Vec<Option<u32>>,
) -> Result<TableRow, String> {
    if node.kind != "tableRow" {
        return Err("Tiptap tables may contain only table rows".into());
    }
    model::no_attrs(&node.attrs, "tableRow")?;
    if node.content.is_empty() {
        return Err("Tiptap table rows require at least one cell".into());
    }
    let header = node.content[0].kind == "tableHeader";
    if node
        .content
        .iter()
        .any(|cell| (cell.kind == "tableHeader") != header)
    {
        return Err("Tiptap table rows cannot mix header and body cells".into());
    }
    let mut cells = Vec::new();
    let mut column = 0usize;
    for node in &node.content {
        while occupancy.get(column).copied().unwrap_or(0) != 0 {
            column += 1;
        }
        let parsed = table_cells::parse(node, context, depth + 1)?;
        let span = parsed.cell.col_span as usize;
        if column + span > 64 {
            return Err("Tiptap tables cannot exceed 64 columns".into());
        }
        occupancy.resize(column + span, 0);
        widths.resize(column + span, None);
        if occupancy[column..column + span]
            .iter()
            .any(|value| *value != 0)
        {
            return Err("Tiptap table cells overlap a row-spanning cell".into());
        }
        for offset in 0..span {
            occupancy[column + offset] = parsed.cell.row_span;
            merge_width(
                &mut widths[column + offset],
                parsed.widths.get(offset).copied().flatten(),
            )?;
        }
        column += span;
        cells.push(parsed.cell);
    }
    Ok(TableRow {
        cells,
        is_header: header,
        repeat_as_header: header,
        ..Default::default()
    })
}

fn merge_width(existing: &mut Option<u32>, width: Option<u32>) -> Result<(), String> {
    if existing.is_some() && width.is_some() && *existing != width {
        return Err("Tiptap table contains conflicting column widths".into());
    }
    if existing.is_none() {
        *existing = width;
    }
    Ok(())
}
