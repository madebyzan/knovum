use pulldown_cmark::{Alignment as MarkdownAlignment, Event, Tag, TagEnd};

use super::super::document::{Alignment, Block, Inline, Table};
use super::Reader;

impl Reader<'_> {
    pub(super) fn list(&mut self, start: Option<u64>, depth: usize) -> Result<Block, String> {
        Self::check_depth(depth)?;
        if start.is_some_and(|value| value > i32::MAX as u64) {
            return Err("Document list start exceeds the supported range".into());
        }
        let mut items = Vec::new();
        while let Some(event) = self.next()? {
            match event {
                Event::Start(Tag::Item) => {
                    items.push(self.blocks(Some(TagEnd::Item), depth + 1)?);
                }
                Event::End(TagEnd::List(ordered)) if ordered == start.is_some() => {
                    return Ok(Block::List { start, items });
                }
                _ => return Err("Unexpected Markdown list event".into()),
            }
        }
        Err("Unclosed Markdown list".into())
    }

    pub(super) fn table(
        &mut self,
        alignments: Vec<MarkdownAlignment>,
        depth: usize,
    ) -> Result<Block, String> {
        Self::check_depth(depth)?;
        if alignments.is_empty() || alignments.len() > 64 {
            return Err("Document tables require between 1 and 64 columns".into());
        }
        let alignments: Vec<_> = alignments.into_iter().map(alignment).collect();
        let columns = alignments.len();
        let mut table = Table {
            alignments,
            header: Vec::new(),
            rows: Vec::new(),
        };
        while let Some(event) = self.next()? {
            match event {
                Event::Start(Tag::TableHead) => {
                    table.header = self.cells(TagEnd::TableHead, columns, depth + 1)?;
                }
                Event::Start(Tag::TableRow) => {
                    table.rows.push(self.cells(TagEnd::TableRow, columns, depth + 1)?);
                }
                Event::End(TagEnd::Table) => return Ok(Block::Table(table)),
                _ => return Err("Unexpected Markdown table event".into()),
            }
        }
        Err("Unclosed Markdown table".into())
    }

    fn cells(&mut self, end: TagEnd, columns: usize, depth: usize) -> Result<Vec<Vec<Inline>>, String> {
        Self::check_depth(depth)?;
        let mut cells = Vec::new();
        while let Some(event) = self.next()? {
            match event {
                Event::Start(Tag::TableCell) => {
                    cells.push(self.inlines(TagEnd::TableCell, depth + 1)?);
                }
                Event::End(tag) if tag == end && cells.len() == columns => return Ok(cells),
                _ => return Err("Invalid Markdown table row".into()),
            }
        }
        Err("Unclosed Markdown table row".into())
    }
}

fn alignment(value: MarkdownAlignment) -> Alignment {
    match value {
        MarkdownAlignment::None | MarkdownAlignment::Left => Alignment::Left,
        MarkdownAlignment::Center => Alignment::Center,
        MarkdownAlignment::Right => Alignment::Right,
    }
}
