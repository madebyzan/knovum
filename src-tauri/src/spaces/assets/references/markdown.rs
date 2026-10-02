use super::{asset_url, AssetReference};
use pulldown_cmark::{Event, Parser, Tag};

pub(super) fn visit_references(text: &str, visit: &mut impl FnMut(AssetReference, &str)) {
    for event in Parser::new(text) {
        if let Event::Start(Tag::Image { dest_url, .. }) = event {
            if let Some(path) = asset_url::managed_path(&dest_url) {
                visit(AssetReference::Embedded, &path);
            }
        }
    }
}

pub(super) fn rewrite_asset_paths<F>(text: &str, rewrite: &mut F) -> Result<String, String>
where
    F: FnMut(&str) -> Result<String, String>,
{
    let mut rewritten = String::with_capacity(text.len());
    let mut cursor = 0;
    for (event, range) in Parser::new(text).into_offset_iter() {
        let Event::Start(Tag::Image {
            dest_url, title, ..
        }) = event
        else {
            continue;
        };
        let Some(asset_path) = asset_url::managed_path(&dest_url) else {
            continue;
        };
        let source = &text[range.clone()];
        let alt_end = image_alt_end(source).ok_or("Invalid Markdown image syntax")?;
        let path = rewrite(&asset_path)?;
        rewritten.push_str(&text[cursor..range.start]);
        rewritten.push_str(&source[..alt_end]);
        rewritten.push_str("(<");
        rewritten.push_str(&path.replace('\\', "\\\\").replace('>', "\\>"));
        rewritten.push('>');
        if !title.is_empty() {
            rewritten.push_str(" \"");
            rewritten.push_str(&title.replace('\\', "\\\\").replace('"', "\\\""));
            rewritten.push('"');
        }
        rewritten.push(')');
        cursor = range.end;
    }
    rewritten.push_str(&text[cursor..]);
    Ok(rewritten)
}

fn image_alt_end(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
    if !source.starts_with("![") {
        return None;
    }
    let mut depth = 1;
    let mut index = 2;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'[' => {
                depth += 1;
                index += 1;
            }
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
                index += 1;
            }
            _ => index += 1,
        }
    }
    None
}
