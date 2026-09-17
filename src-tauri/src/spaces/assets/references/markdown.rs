use super::{asset_url, AssetReference};

pub(super) fn visit_references(text: &str, visit: &mut impl FnMut(AssetReference, &str)) {
    for event in pulldown_cmark::Parser::new(text) {
        if let pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image { dest_url, .. }) = event {
            let path = asset_url::asset_path(&dest_url);
            visit(AssetReference::Embedded, &path);
        }
    }
}

pub(super) fn rewrite_asset_paths<F>(text: &str, rewrite: &mut F) -> String
where
    F: FnMut(&str) -> String,
{
    let mut result = String::with_capacity(text.len());
    let mut cursor = 0;
    while let Some(start) = text[cursor..].find("![") {
        let actual_start = cursor + start;
        result.push_str(&text[cursor..actual_start]);
        if let Some(close_bracket) = text[actual_start..].find("](") {
            let paren_start = actual_start + close_bracket + 2;
            if let Some(close_paren) = text[paren_start..].find(')') {
                let paren_end = paren_start + close_paren;
                let path = asset_url::asset_path(&text[paren_start..paren_end]);
                result.push_str(&text[actual_start..paren_start]);
                result.push_str(&rewrite(&path));
                result.push(')');
                cursor = paren_end + 1;
                continue;
            }
        }
        result.push_str("![");
        cursor = actual_start + 2;
    }
    result.push_str(&text[cursor..]);
    result
}
