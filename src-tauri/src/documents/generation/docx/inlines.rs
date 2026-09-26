use docx_rs::{BreakType, Hyperlink, HyperlinkType, Paragraph, Run};

use super::super::document::Inline;

#[derive(Clone, Copy, Default)]
struct Marks {
    bold: bool,
    italic: bool,
    strike: bool,
}

pub(super) fn paragraph(content: &[Inline]) -> Result<Paragraph, String> {
    append(Paragraph::new(), content, Marks::default())
}

pub(super) fn header_paragraph(content: &[Inline]) -> Result<Paragraph, String> {
    append(
        Paragraph::new(),
        content,
        Marks {
            bold: true,
            ..Marks::default()
        },
    )
}

fn append(mut paragraph: Paragraph, content: &[Inline], marks: Marks) -> Result<Paragraph, String> {
    for inline in content {
        match inline {
            Inline::Strong(children) => {
                paragraph = append(
                    paragraph,
                    children,
                    Marks {
                        bold: true,
                        ..marks
                    },
                )?;
            }
            Inline::Emphasis(children) => {
                paragraph = append(
                    paragraph,
                    children,
                    Marks {
                        italic: true,
                        ..marks
                    },
                )?;
            }
            Inline::Strike(children) => {
                paragraph = append(
                    paragraph,
                    children,
                    Marks {
                        strike: true,
                        ..marks
                    },
                )?;
            }
            Inline::Link { url, content } => {
                super::super::links::validate(url)?;
                let mut link = Hyperlink::new(url, HyperlinkType::External);
                for run in runs(content, marks)? {
                    link = link.add_run(run.color("0563C1").underline("single"));
                }
                paragraph = paragraph.add_hyperlink(link);
            }
            inline => paragraph = paragraph.add_run(leaf(inline, marks)?),
        }
    }
    Ok(paragraph)
}

fn runs(content: &[Inline], marks: Marks) -> Result<Vec<Run>, String> {
    let mut output = Vec::new();
    for inline in content {
        let children = match inline {
            Inline::Strong(children) => Some((
                children,
                Marks {
                    bold: true,
                    ..marks
                },
            )),
            Inline::Emphasis(children) => Some((
                children,
                Marks {
                    italic: true,
                    ..marks
                },
            )),
            Inline::Strike(children) => Some((
                children,
                Marks {
                    strike: true,
                    ..marks
                },
            )),
            _ => None,
        };
        if let Some((children, marks)) = children {
            output.extend(runs(children, marks)?);
        } else {
            output.push(leaf(inline, marks)?);
        }
    }
    Ok(output)
}

fn leaf(inline: &Inline, marks: Marks) -> Result<Run, String> {
    let mut run = match inline {
        Inline::Text(text) => text_run(text),
        Inline::Code(text) => text_run(text).style("KnovumCode"),
        Inline::Task(checked) => Run::new().add_text(if *checked { "[x] " } else { "[ ] " }),
        Inline::SoftBreak => Run::new().add_text(" "),
        Inline::HardBreak => Run::new().add_break(BreakType::TextWrapping),
        Inline::Image { .. } => {
            return Err(
                "DOCX image export is not implemented; use plain text or remove images".into(),
            );
        }
        _ => return Err("Unexpected nested inline in DOCX export".into()),
    };
    if marks.bold {
        run = run.bold();
    }
    if marks.italic {
        run = run.italic();
    }
    if marks.strike {
        run = run.strike();
    }
    Ok(run)
}

pub(super) fn text_run(text: &str) -> Run {
    let mut run = Run::new();
    let mut buffer = String::new();
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    for c in normalized.chars() {
        if matches!(c, '\n' | '\t') {
            run = run.add_text(std::mem::take(&mut buffer));
            run = if c == '\n' {
                run.add_break(BreakType::TextWrapping)
            } else {
                run.add_tab()
            };
        } else {
            buffer.push(c);
        }
    }
    run.add_text(buffer)
}

pub(super) fn code_paragraph(text: &str) -> Paragraph {
    Paragraph::new().add_run(text_run(text.strip_suffix('\n').unwrap_or(text)).style("KnovumCode"))
}
