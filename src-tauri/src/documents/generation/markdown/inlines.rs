use pulldown_cmark::{Event, Tag, TagEnd};

use super::super::document::Inline;
use super::Reader;

impl<'a> Reader<'a> {
    pub(super) fn inlines(&mut self, end: TagEnd, depth: usize) -> Result<Vec<Inline>, String> {
        Self::check_depth(depth)?;
        let mut output = Vec::new();
        while let Some(event) = self.next()? {
            if let Event::End(tag) = event {
                return if tag == end {
                    Ok(output)
                } else {
                    Err("Unexpected Markdown inline boundary".into())
                };
            }
            self.push_inline(event, depth + 1, &mut output)?;
        }
        Err("Unclosed Markdown inline element".into())
    }

    /// One event, as the content it adds. Raw HTML is the event that adds more than one thing -
    /// or nothing - so inline content is built through here rather than one value at a time.
    pub(super) fn push_inline(
        &mut self,
        event: Event<'a>,
        depth: usize,
        output: &mut Vec<Inline>,
    ) -> Result<(), String> {
        match event {
            Event::Html(value) | Event::InlineHtml(value) => super::html::read(&value, output),
            event => output.push(self.inline(event, depth)?),
        }
        Ok(())
    }

    pub(super) fn inline(&mut self, event: Event<'a>, depth: usize) -> Result<Inline, String> {
        Self::check_depth(depth)?;
        Ok(match event {
            Event::Text(value) => Inline::Text(value.into_string()),
            Event::Code(value) => Inline::Code(value.into_string()),
            Event::SoftBreak => Inline::SoftBreak,
            Event::HardBreak => Inline::HardBreak,
            Event::TaskListMarker(checked) => Inline::Task(checked),
            Event::Start(Tag::Strong) => Inline::Strong(self.inlines(TagEnd::Strong, depth + 1)?),
            Event::Start(Tag::Emphasis) => {
                Inline::Emphasis(self.inlines(TagEnd::Emphasis, depth + 1)?)
            }
            Event::Start(Tag::Strikethrough) => {
                Inline::Strike(self.inlines(TagEnd::Strikethrough, depth + 1)?)
            }
            Event::Start(Tag::Link { dest_url, .. }) => Inline::Link {
                url: dest_url.into_string(),
                content: self.inlines(TagEnd::Link, depth + 1)?,
            },
            Event::Start(Tag::Image { dest_url, .. }) => Inline::Image {
                destination: dest_url.into_string(),
                alt: self.inlines(TagEnd::Image, depth + 1)?,
            },
            _ => return Err("Unsupported Markdown element in document export".into()),
        })
    }
}
