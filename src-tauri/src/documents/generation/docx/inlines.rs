use docx_rs::{
    AlignmentType, BreakType, Hyperlink, HyperlinkType, Paragraph, Pic, Run, VertAlignType,
};

use super::super::{
    document::Inline,
    images::Images,
    math::{self, MathScript},
};

#[derive(Clone, Copy, Default)]
struct Marks {
    bold: bool,
    italic: bool,
    strike: bool,
}

pub(super) fn paragraph(content: &[Inline], images: &Images) -> Result<Paragraph, String> {
    append(Paragraph::new(), content, Marks::default(), images)
}

pub(super) fn header_paragraph(content: &[Inline], images: &Images) -> Result<Paragraph, String> {
    append(
        Paragraph::new(),
        content,
        Marks {
            bold: true,
            ..Marks::default()
        },
        images,
    )
}

fn append(
    mut paragraph: Paragraph,
    content: &[Inline],
    marks: Marks,
    images: &Images,
) -> Result<Paragraph, String> {
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
                    images,
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
                    images,
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
                    images,
                )?;
            }
            Inline::Link { url, content } => {
                super::super::links::validate(url)?;
                let mut link = Hyperlink::new(url, HyperlinkType::External);
                for run in runs(content, marks, images)? {
                    link = link.add_run(run.color("0563C1").underline("single"));
                }
                paragraph = paragraph.add_hyperlink(link);
            }
            Inline::Math(latex) | Inline::DisplayMath(latex) => {
                for run in math_runs(latex, marks) {
                    paragraph = paragraph.add_run(run);
                }
            }
            inline => paragraph = paragraph.add_run(leaf(inline, marks, images)?),
        }
    }
    Ok(paragraph)
}

fn runs(content: &[Inline], marks: Marks, images: &Images) -> Result<Vec<Run>, String> {
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
            output.extend(runs(children, marks, images)?);
        } else if let Inline::Math(latex) | Inline::DisplayMath(latex) = inline {
            output.extend(math_runs(latex, marks));
        } else {
            output.push(leaf(inline, marks, images)?);
        }
    }
    Ok(output)
}

fn leaf(inline: &Inline, marks: Marks, images: &Images) -> Result<Run, String> {
    let mut run = match inline {
        Inline::Text(text) => text_run(text),
        Inline::Code(text) => text_run(text).style("KnovumCode"),
        Inline::Task(checked) => Run::new().add_text(if *checked { "[x] " } else { "[ ] " }),
        Inline::SoftBreak => Run::new().add_text(" "),
        Inline::HardBreak => Run::new().add_break(BreakType::TextWrapping),
        Inline::Image { destination, .. } => {
            let image = images.get(destination)?;
            let (width, height) = super::image_layout::pixel_size(image.width, image.height);
            let pic = Pic::new_with_dimensions(
                images.clone_bytes(destination)?,
                image.width,
                image.height,
            )
            .size(width as u32, height as u32);
            Run::new().add_image(pic)
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

pub(super) fn math_block_paragraph(latex: &str) -> Paragraph {
    let mut paragraph = Paragraph::new().align(AlignmentType::Center);
    for run in math_runs(latex, Marks::default()) {
        paragraph = paragraph.add_run(run);
    }
    paragraph
}

fn math_runs(latex: &str, marks: Marks) -> Vec<Run> {
    math::to_runs(latex)
        .into_iter()
        .map(|part| {
            let mut run = text_run(&part.text);
            if marks.bold || part.bold {
                run = run.bold();
            }
            if marks.italic || part.italic {
                run = run.italic();
            }
            if marks.strike {
                run = run.strike();
            }
            if let Some(script) = part.script {
                run.run_property = run.run_property.vert_align(match script {
                    MathScript::Subscript => VertAlignType::SubScript,
                    MathScript::Superscript => VertAlignType::SuperScript,
                });
            }
            run
        })
        .collect()
}

pub(super) fn code_paragraph(text: &str) -> Paragraph {
    Paragraph::new().add_run(text_run(text.strip_suffix('\n').unwrap_or(text)).style("KnovumCode"))
}
