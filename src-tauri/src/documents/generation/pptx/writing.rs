use office_oxide::{
    ir::{Element, InlineContent, Section, Table},
    pptx::write::{Run, SlideData},
};

pub(super) fn slide(slide: &mut SlideData, section: &Section) -> Result<(), String> {
    if let Some(title) = &section.title {
        slide.set_title(title);
    }
    for element in &section.elements {
        match element {
            Element::Heading(heading)
                if section.title.as_deref()
                    == Some(super::paragraphs::plain(&heading.content).as_str()) => {}
            Element::Heading(heading) => {
                slide.add_rich_text(&runs(&heading.content)?);
            }
            Element::Paragraph(paragraph) => {
                slide
                    .add_rich_text_aligned(&runs(&paragraph.content)?, paragraph.alignment.clone());
            }
            Element::Table(table) => {
                slide.add_table(table_rows(table)?);
            }
            Element::Image(image) => {
                let width = image
                    .display_width_emu
                    .ok_or("Presentation image width is missing")?;
                let height = image
                    .display_height_emu
                    .ok_or("Presentation image height is missing")?;
                let x = (12_192_000_u64.saturating_sub(width) / 2) as i64;
                let y = 1_500_000 + (4_500_000_u64.saturating_sub(height) / 2) as i64;
                slide.add_image_with_alt(
                    image
                        .data
                        .clone()
                        .ok_or("Presentation image bytes are missing")?,
                    image
                        .format
                        .clone()
                        .ok_or("Presentation image format is missing")?,
                    x,
                    y,
                    width,
                    height,
                    image.alt_text.clone(),
                );
            }
            Element::ThematicBreak => {
                slide.add_text("────────────────────────");
            }
            _ => return Err("Presentation pagination produced unsupported content".into()),
        }
    }
    Ok(())
}

fn runs(content: &[InlineContent]) -> Result<Vec<Run>, String> {
    content
        .iter()
        .map(|inline| match inline {
            InlineContent::LineBreak => Ok(Run::line_break()),
            InlineContent::Text(span) => {
                if let Some(url) = &span.hyperlink {
                    super::super::links::validate(url)?;
                }
                Ok(Run {
                    text: span.text.clone(),
                    bold: span.bold,
                    italic: span.italic,
                    underline: span.underline.is_some(),
                    strikethrough: span.strikethrough,
                    font_size_pt: Some(
                        span.font_size_half_pt
                            .map_or(20.0, |size| f64::from(size) / 2.0),
                    ),
                    font_name: span.font_name.clone(),
                    color: span
                        .color
                        .map(|rgb| format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])),
                    hyperlink: span.hyperlink.clone(),
                    hyperlink_tooltip: span.hyperlink_tooltip.clone(),
                    ..Default::default()
                })
            }
            _ => Err("Presentation contains unsupported inline content".into()),
        })
        .collect()
}

fn table_rows(table: &Table) -> Result<Vec<Vec<Vec<Run>>>, String> {
    table
        .rows
        .iter()
        .map(|row| {
            row.cells
                .iter()
                .map(|cell| {
                    let mut cell_runs = Vec::new();
                    for (index, element) in cell.content.iter().enumerate() {
                        if index > 0 {
                            cell_runs.push(Run::line_break());
                        }
                        match element {
                            Element::Paragraph(paragraph) => {
                                cell_runs.extend(runs(&paragraph.content)?)
                            }
                            _ => {
                                return Err(
                                    "Presentation table contains unsupported cell content".into()
                                )
                            }
                        }
                    }
                    if row.is_header {
                        for run in &mut cell_runs {
                            run.bold = true;
                        }
                    }
                    Ok(cell_runs)
                })
                .collect()
        })
        .collect()
}
