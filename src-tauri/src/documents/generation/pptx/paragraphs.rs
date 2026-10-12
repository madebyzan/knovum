use office_oxide::ir::{Element, InlineContent, Paragraph, TextSpan};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use super::pagination::{Paginator, BODY_LINES, LINE_WIDTH};

pub(super) struct Chunk {
    pub content: Vec<InlineContent>,
    pub lines: usize,
}

pub(super) fn plain(content: &[InlineContent]) -> String {
    content
        .iter()
        .map(|item| match item {
            InlineContent::Text(span) => span.text.as_str(),
            InlineContent::LineBreak => "\n",
            _ => "",
        })
        .collect()
}

impl Paginator {
    pub fn paragraph(&mut self, paragraph: &Paragraph, marker: Option<&str>) -> Result<(), String> {
        let mut content = Vec::new();
        if let Some(marker) = marker {
            content.push(InlineContent::Text(TextSpan::plain(marker)));
        }
        content.extend(paragraph.content.clone());
        for chunk in split(&content, LINE_WIDTH, BODY_LINES - 1)? {
            self.reserve(chunk.lines + 1)?;
            self.current.elements.push(Element::Paragraph(Paragraph {
                content: chunk.content,
                alignment: paragraph.alignment.clone(),
                ..Default::default()
            }));
            self.used += chunk.lines + 1;
        }
        Ok(())
    }
}

pub(super) fn split(
    content: &[InlineContent],
    line_width: usize,
    max_lines: usize,
) -> Result<Vec<Chunk>, String> {
    let mut result = Vec::new();
    let mut chunk = Vec::new();
    let mut lines = 1;
    let mut width = 0;
    for inline in content {
        match inline {
            InlineContent::Text(span) => {
                if let Some(url) = &span.hyperlink {
                    super::super::links::validate(url)?;
                }
                let mut pending = String::new();
                for grapheme in span.text.graphemes(true) {
                    if grapheme.contains('\n') || grapheme == "\r" {
                        append_span(&mut chunk, span, std::mem::take(&mut pending));
                        if lines >= max_lines {
                            result.push(Chunk {
                                content: std::mem::take(&mut chunk),
                                lines,
                            });
                            lines = 1;
                        } else {
                            chunk.push(InlineContent::LineBreak);
                            lines += 1;
                        }
                        width = 0;
                        continue;
                    }
                    let units = grapheme.width_cjk().max(1);
                    if width + units > line_width {
                        if lines >= max_lines {
                            append_span(&mut chunk, span, std::mem::take(&mut pending));
                            result.push(Chunk {
                                content: std::mem::take(&mut chunk),
                                lines,
                            });
                            lines = 1;
                        } else {
                            lines += 1;
                        }
                        width = 0;
                    }
                    pending.push_str(grapheme);
                    width += units;
                }
                append_span(&mut chunk, span, pending);
            }
            InlineContent::LineBreak => {
                if lines >= max_lines {
                    result.push(Chunk {
                        content: std::mem::take(&mut chunk),
                        lines,
                    });
                    lines = 1;
                } else {
                    lines += 1;
                    chunk.push(InlineContent::LineBreak);
                }
                width = 0;
            }
            _ => return Err("Presentation contains an unsupported text reference".into()),
        }
    }
    if !chunk.is_empty() {
        result.push(Chunk {
            content: chunk,
            lines,
        });
    }
    if result.is_empty() {
        result.push(Chunk {
            content: vec![],
            lines: 1,
        });
    }
    Ok(result)
}

fn append_span(output: &mut Vec<InlineContent>, original: &TextSpan, text: String) {
    if text.is_empty() {
        return;
    }
    let mut span = original.clone();
    span.text = text;
    span.font_size_half_pt = Some(40);
    if span.font_name.is_none() {
        span.font_name = Some(crate::desktop::system_font::SYSTEM_SANS_FAMILY.into());
    }
    output.push(InlineContent::Text(span));
}
