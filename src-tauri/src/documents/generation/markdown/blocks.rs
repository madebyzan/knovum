use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Tag, TagEnd};

use super::super::document::{Block, Inline};
use super::Reader;

impl Reader<'_> {
    pub(super) fn blocks(
        &mut self,
        end: Option<TagEnd>,
        depth: usize,
    ) -> Result<Vec<Block>, String> {
        Self::check_depth(depth)?;
        let mut blocks = Vec::new();
        let mut loose = Vec::new();
        while let Some(event) = self.next()? {
            if let Event::End(tag) = event {
                if Some(tag) != end {
                    return Err("Unexpected Markdown block boundary".into());
                }
                flush(&mut blocks, &mut loose);
                return Ok(blocks);
            }
            let block = match event {
                Event::Start(Tag::Paragraph) => {
                    Block::Paragraph(self.inlines(TagEnd::Paragraph, depth + 1)?)
                }
                Event::Start(Tag::Heading { level, .. }) => Block::Heading {
                    level: heading_level(level),
                    content: self.inlines(TagEnd::Heading(level), depth + 1)?,
                },
                Event::Start(Tag::BlockQuote(kind)) => {
                    Block::Quote(self.blocks(Some(TagEnd::BlockQuote(kind)), depth + 1)?)
                }
                Event::Start(Tag::CodeBlock(kind)) => self.code_block(kind)?,
                Event::Start(Tag::List(start)) => self.list(start, depth + 1)?,
                Event::Start(Tag::Table(alignments)) => self.table(alignments, depth + 1)?,
                Event::Rule => Block::Rule,
                // Markdown hands a whole block to HTML to lay something out. The layout is not
                // document content, but what it wraps is.
                Event::Start(Tag::HtmlBlock) => {
                    self.html_block(depth + 1, &mut loose)?;
                    continue;
                }
                event => {
                    self.push_inline(event, depth + 1, &mut loose)?;
                    continue;
                }
            };
            flush(&mut blocks, &mut loose);
            blocks.push(block);
        }
        if end.is_some() {
            return Err("Unclosed Markdown block".into());
        }
        flush(&mut blocks, &mut loose);
        Ok(blocks)
    }

    fn html_block(&mut self, depth: usize, output: &mut Vec<Inline>) -> Result<(), String> {
        Self::check_depth(depth)?;
        while let Some(event) = self.next()? {
            match event {
                Event::End(TagEnd::HtmlBlock) => return Ok(()),
                event => self.push_inline(event, depth, output)?,
            }
        }
        Err("Unclosed Markdown HTML block".into())
    }

    fn code_block(&mut self, kind: CodeBlockKind<'_>) -> Result<Block, String> {
        let language = match kind {
            CodeBlockKind::Fenced(info) => info.split_whitespace().next().unwrap_or("").to_owned(),
            CodeBlockKind::Indented => String::new(),
        };
        let mut text = String::new();
        while let Some(event) = self.next()? {
            match event {
                Event::Text(value) => text.push_str(&value),
                Event::End(TagEnd::CodeBlock) => return Ok(Block::Code { language, text }),
                _ => return Err("Unexpected Markdown code-block event".into()),
            }
        }
        Err("Unclosed Markdown code block".into())
    }
}

fn flush(blocks: &mut Vec<Block>, loose: &mut Vec<Inline>) {
    if !loose.is_empty() {
        blocks.push(Block::Paragraph(std::mem::take(loose)));
    }
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}
