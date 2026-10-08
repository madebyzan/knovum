use serde_json::{json, Value};

use super::contracts::JsonCanvasNodeV1;
use crate::spaces::assets::PreparedAsset;

pub(super) fn payload(
    node: &JsonCanvasNodeV1,
    note_title: &str,
    asset: Option<PreparedAsset>,
) -> Result<Value, String> {
    match node {
        JsonCanvasNodeV1::Text(node) => {
            let first = node
                .text
                .split('\n')
                .next()
                .ok_or("Invalid note text")?
                .trim_end_matches('\r');
            let title: String = first
                .chars()
                .scan(0, |units, character| {
                    *units += character.len_utf16();
                    (*units <= 80).then_some(character)
                })
                .collect();
            let title = if title.is_empty() { note_title } else { &title };
            let paragraphs: Vec<_> = node
                .text
                .split('\n')
                .map(|line| {
                    let line = line.trim_end_matches('\r');
                    if line.is_empty() {
                        json!({"type":"paragraph"})
                    } else {
                        json!({"type":"paragraph","content":[{"type":"text","text":line}]})
                    }
                })
                .collect();
            Ok(
                json!({"kind":"note","title":title,"plainText":node.text,"tiptapContent":{"type":"doc","content":paragraphs}}),
            )
        }
        JsonCanvasNodeV1::Link(node) => {
            let url = website_url(&node.url)?;
            if !matches!(url.scheme(), "http" | "https") {
                return Err("JSON Canvas link must use HTTP or HTTPS".into());
            }
            let host = url.host_str().ok_or("JSON Canvas link has no host")?;
            let title = host.strip_prefix("www.").unwrap_or(host);
            Ok(
                json!({"kind":"website","title":title,"siteName":title,"url":url.as_str(),"metadataState":"idle"}),
            )
        }
        JsonCanvasNodeV1::File(_) => crate::spaces::assets::node_payload::from_import(
            asset.ok_or("JSON Canvas file has no prepared resource")?,
        ),
        JsonCanvasNodeV1::Group(_) => Err("A visual group is not a content node".into()),
    }
}

fn website_url(value: &str) -> Result<reqwest::Url, String> {
    let input = value.trim();
    if input.is_empty()
        || input.encode_utf16().count() > 4096
        || input.chars().any(char::is_whitespace)
    {
        return Err("JSON Canvas link is not an HTTP URL".into());
    }
    let candidate = if input.contains("://") {
        input.to_owned()
    } else if input.contains('.') {
        format!("https://{input}")
    } else {
        return Err("JSON Canvas link is not an HTTP URL".into());
    };
    reqwest::Url::parse(&candidate).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
