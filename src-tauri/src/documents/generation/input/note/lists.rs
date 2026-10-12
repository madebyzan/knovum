use office_oxide::ir::{Element, List, ListItem, ListStyle, Paragraph};

use super::{blocks, inlines, model, Context};
use model::Node;

pub(super) fn parse(
    node: &Node,
    context: &Context<'_>,
    depth: usize,
    level: u8,
) -> Result<Element, String> {
    if depth > 64 || level > 64 {
        return Err("Document export nesting exceeds 64 levels".into());
    }
    let (ordered, start, item_kind) = match node.kind.as_str() {
        "bulletList" => {
            model::no_attrs(&node.attrs, "bulletList")?;
            (false, None, "listItem")
        }
        "orderedList" => {
            model::allowed_attrs(&node.attrs, "orderedList", &["start", "type"])?;
            let start = model::u32_attr(&node.attrs, "start", 1)?;
            if start == 0 || start > i32::MAX as u32 {
                return Err("Tiptap ordered-list start is outside the supported range".into());
            }
            validate_ordered_type(node)?;
            (true, Some(start), "listItem")
        }
        "taskList" => {
            model::no_attrs(&node.attrs, "taskList")?;
            (false, None, "taskItem")
        }
        _ => return Err("Unsupported Tiptap list node".into()),
    };
    if node.content.is_empty() {
        return Err("Tiptap lists require at least one item".into());
    }
    let items = node
        .content
        .iter()
        .map(|item| list_item(item, item_kind, context, depth + 1, level))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Element::List(List {
        ordered,
        items,
        start_number: start,
        style: Some(if ordered {
            ListStyle::Decimal
        } else {
            ListStyle::Bullet
        }),
        level,
        ..Default::default()
    }))
}

fn list_item(
    node: &Node,
    expected: &str,
    context: &Context<'_>,
    depth: usize,
    level: u8,
) -> Result<ListItem, String> {
    if node.kind != expected {
        return Err(format!("Tiptap list requires {expected} children"));
    }
    let checked = if expected == "taskItem" {
        model::allowed_attrs(&node.attrs, "taskItem", &["checked"])?;
        Some(model::bool_attr(&node.attrs, "checked", false)?)
    } else {
        model::no_attrs(&node.attrs, "listItem")?;
        None
    };
    let mut content = Vec::new();
    let mut nested = None;
    for child in &node.content {
        if matches!(
            child.kind.as_str(),
            "bulletList" | "orderedList" | "taskList"
        ) {
            if nested.is_some() {
                return Err("Tiptap list items cannot contain multiple nested lists".into());
            }
            let mut element = parse(child, context, depth + 1, level + 1)?;
            let Element::List(list) = &mut element else {
                unreachable!()
            };
            nested = Some(std::mem::take(list));
        } else {
            if nested.is_some() {
                return Err("Tiptap nested lists must follow their item content".into());
            }
            content.extend(blocks::parse(
                std::slice::from_ref(child),
                context,
                depth + 1,
            )?);
        }
    }
    if content.is_empty() {
        return Err("Tiptap list items require visible content".into());
    }
    if let Some(checked) = checked {
        prepend_task_state(&mut content, checked);
    }
    Ok(ListItem { content, nested })
}

fn prepend_task_state(content: &mut Vec<Element>, checked: bool) {
    let marker = inlines::plain(if checked { "[x] " } else { "[ ] " });
    if let Some(Element::Paragraph(paragraph)) = content.first_mut() {
        paragraph.content.insert(0, marker);
    } else {
        content.insert(
            0,
            Element::Paragraph(Paragraph {
                content: vec![marker],
                ..Default::default()
            }),
        );
    }
}

fn validate_ordered_type(node: &Node) -> Result<(), String> {
    match model::string_attr(&node.attrs, "type")? {
        None | Some("1") => Ok(()),
        Some(_) => Err("Only decimal Tiptap ordered lists are supported".into()),
    }
}
