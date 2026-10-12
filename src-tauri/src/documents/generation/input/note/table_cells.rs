use office_oxide::ir::{ParagraphAlignment, TableCell};
use serde_json::Value;

use super::{blocks, colors, model, Context};
use model::Node;

pub(super) struct ParsedCell {
    pub cell: TableCell,
    pub widths: Vec<Option<u32>>,
}

pub(super) fn parse(
    node: &Node,
    context: &Context<'_>,
    depth: usize,
) -> Result<ParsedCell, String> {
    if !matches!(node.kind.as_str(), "tableCell" | "tableHeader") {
        return Err("Tiptap table rows may contain only cells".into());
    }
    model::allowed_attrs(
        &node.attrs,
        &node.kind,
        &[
            "colspan",
            "rowspan",
            "colwidth",
            "align",
            "textAlign",
            "background",
            "backgroundColor",
        ],
    )?;
    let col_span = model::u32_attr(&node.attrs, "colspan", 1)?;
    let row_span = model::u32_attr(&node.attrs, "rowspan", 1)?;
    if col_span == 0 || col_span > 64 || row_span == 0 || row_span > 10_000 {
        return Err("Tiptap table cell span is outside the supported range".into());
    }
    if node.content.is_empty() {
        return Err("Tiptap table cells require block content".into());
    }
    let align = paired_string(node, "align", "textAlign")?;
    let background = paired_string(node, "background", "backgroundColor")?;
    Ok(ParsedCell {
        cell: TableCell {
            content: blocks::parse(&node.content, context, depth + 1)?,
            col_span,
            row_span,
            background_color: background.map(colors::parse).transpose()?,
            text_align: align.map(alignment).transpose()?,
            ..Default::default()
        },
        widths: column_widths(node, col_span)?,
    })
}

fn column_widths(node: &Node, span: u32) -> Result<Vec<Option<u32>>, String> {
    let Some(value) = node.attrs.get("colwidth") else {
        return Ok(vec![None; span as usize]);
    };
    if value.is_null() {
        return Ok(vec![None; span as usize]);
    }
    let values = value
        .as_array()
        .ok_or("Tiptap colwidth must be an array or null")?;
    if values.len() != span as usize {
        return Err("Tiptap colwidth must match colspan".into());
    }
    values
        .iter()
        .map(|value| match value {
            Value::Null => Ok(None),
            Value::Number(number) => number
                .as_u64()
                .filter(|width| (1..=16_384).contains(width))
                .map(|width| Some(width as u32 * 15))
                .ok_or_else(|| "Tiptap column width is outside the supported range".into()),
            _ => Err("Tiptap column width must be an integer or null".into()),
        })
        .collect()
}

fn paired_string<'a>(node: &'a Node, first: &str, second: &str) -> Result<Option<&'a str>, String> {
    let a = model::string_attr(&node.attrs, first)?;
    let b = model::string_attr(&node.attrs, second)?;
    if a.is_some() && b.is_some() && a != b {
        return Err(format!("Tiptap cell has conflicting {first} and {second}"));
    }
    Ok(a.or(b))
}

fn alignment(value: &str) -> Result<ParagraphAlignment, String> {
    match value {
        "left" => Ok(ParagraphAlignment::Left),
        "center" => Ok(ParagraphAlignment::Center),
        "right" => Ok(ParagraphAlignment::Right),
        "justify" => Ok(ParagraphAlignment::Justify),
        _ => Err("Unsupported Tiptap table-cell alignment".into()),
    }
}
