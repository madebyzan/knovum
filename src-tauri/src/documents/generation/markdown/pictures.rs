use std::collections::BTreeSet;

use super::super::document::{Block, Inline, Table};

/// Keeps only the pictures this Space holds.
///
/// A conversion embeds bytes, so it can only embed what was imported. A reference to anything
/// else - a badge served over the network, a path outside the Space - reads as its alt text,
/// which is what that reference says in a document nobody can fetch from.
pub(super) fn held(blocks: &mut Vec<Block>, available: &BTreeSet<&str>) {
    for block in blocks.iter_mut() {
        match block {
            Block::Paragraph(content) | Block::Heading { content, .. } => {
                resolve(content, available);
            }
            Block::Quote(children) => held(children, available),
            Block::List { items, .. } => {
                for item in items.iter_mut() {
                    held(item, available);
                }
            }
            Block::Table(table) => resolve_table(table, available),
            Block::Code { .. } | Block::Rule => {}
        }
    }
    // A paragraph that held nothing but unavailable pictures has nothing left to say.
    blocks.retain(|block| !matches!(block, Block::Paragraph(content) if content.is_empty()));
}

fn resolve_table(table: &mut Table, available: &BTreeSet<&str>) {
    for cell in table.header.iter_mut() {
        resolve(cell, available);
    }
    for row in table.rows.iter_mut() {
        for cell in row.iter_mut() {
            resolve(cell, available);
        }
    }
}

fn resolve(content: &mut Vec<Inline>, available: &BTreeSet<&str>) {
    let mut output = Vec::with_capacity(content.len());
    for inline in std::mem::take(content) {
        match inline {
            Inline::Image { destination, alt } if !available.contains(destination.as_str()) => {
                output.extend(resolved(alt, available));
            }
            Inline::Strong(children) => output.push(Inline::Strong(resolved(children, available))),
            Inline::Emphasis(children) => {
                output.push(Inline::Emphasis(resolved(children, available)));
            }
            Inline::Strike(children) => output.push(Inline::Strike(resolved(children, available))),
            Inline::Link { url, content } => output.push(Inline::Link {
                url,
                content: resolved(content, available),
            }),
            inline => output.push(inline),
        }
    }
    *content = output;
}

fn resolved(mut content: Vec<Inline>, available: &BTreeSet<&str>) -> Vec<Inline> {
    resolve(&mut content, available);
    content
}
