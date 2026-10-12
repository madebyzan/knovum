use office_oxide::ir::{
    CodeBlock, DocumentIR, Element, InlineContent, List, Paragraph, Section, TextSpan,
};

pub(super) const BODY_LINES: usize = 10;
pub(super) const LINE_WIDTH: usize = 40;

pub(super) struct Paginator {
    pub slides: Vec<Section>,
    pub current: Section,
    pub used: usize,
}

pub(super) fn paginate(ir: &DocumentIR) -> Result<DocumentIR, String> {
    let title = ir.metadata.title.clone().unwrap_or_default();
    let (cover_title, cover_body) = super::titles::split(&title);
    let mut paginator = Paginator {
        slides: vec![],
        current: Section {
            title: Some(cover_title),
            ..Default::default()
        },
        used: 0,
    };
    if !title.is_empty() {
        if !cover_body.is_empty() {
            paginator.paragraph(
                &Paragraph {
                    content: vec![InlineContent::Text(TextSpan::plain(cover_body))],
                    ..Default::default()
                },
                None,
            )?;
        }
        paginator.flush(true)?;
    }
    for (index, section) in ir.sections.iter().enumerate() {
        let start = usize::from(index == 0 && !title.is_empty() && section.elements.first().is_some_and(|element| {
            matches!(element, Element::Heading(value) if super::paragraphs::plain(&value.content).trim() == title)
        }));
        paginator.elements(&section.elements[start..], 0)?;
    }
    paginator.flush(false)?;
    if paginator.slides.is_empty() {
        paginator.flush(true)?;
    }
    let mut metadata = ir.metadata.clone();
    metadata.format = office_oxide::DocumentFormat::Pptx;
    Ok(DocumentIR {
        metadata,
        sections: paginator.slides,
        ..Default::default()
    })
}

impl Paginator {
    pub fn flush(&mut self, force: bool) -> Result<(), String> {
        if !force && self.current.elements.is_empty() {
            return Ok(());
        }
        if self.slides.len() >= 100 {
            return Err("presentation-slide-limit: presentation exceeds 100 slides".into());
        }
        let title = self.current.title.clone();
        self.slides.push(std::mem::replace(
            &mut self.current,
            Section {
                title,
                ..Default::default()
            },
        ));
        self.used = 0;
        Ok(())
    }

    pub fn elements(&mut self, elements: &[Element], level: u8) -> Result<(), String> {
        for element in elements {
            match element {
                Element::Heading(value) => {
                    self.flush(false)?;
                    let full_title = super::paragraphs::plain(&value.content);
                    let (title, overflow) = super::titles::split(&full_title);
                    self.current.title = Some(title.clone());
                    self.current
                        .elements
                        .push(Element::Heading(office_oxide::ir::Heading {
                            level: value.level,
                            content: vec![InlineContent::Text(TextSpan::plain(title))],
                            ..Default::default()
                        }));
                    if !overflow.is_empty() {
                        self.paragraph(
                            &Paragraph {
                                content: vec![InlineContent::Text(TextSpan {
                                    text: overflow,
                                    bold: true,
                                    ..Default::default()
                                })],
                                ..Default::default()
                            },
                            None,
                        )?;
                    }
                }
                Element::Paragraph(value) => self.paragraph(value, None)?,
                Element::CodeBlock(value) => self.code(value)?,
                Element::List(value) => self.list(value, level)?,
                Element::Table(value) => self.table(value)?,
                Element::Image(value) => {
                    if self
                        .current
                        .elements
                        .iter()
                        .any(|element| !matches!(element, Element::Heading(_)))
                    {
                        self.flush(false)?;
                    } else {
                        self.current.elements.clear();
                    }
                    let data = super::super::images::decode(
                        value
                            .data
                            .as_deref()
                            .ok_or("Presentation image bytes are missing")?,
                    )?;
                    let mut image = value.clone();
                    let scale = (10_000_000.0 / f64::from(data.width))
                        .min(4_500_000.0 / f64::from(data.height));
                    image.display_width_emu = Some((f64::from(data.width) * scale) as u64);
                    image.display_height_emu = Some((f64::from(data.height) * scale) as u64);
                    image.data = Some(data.bytes);
                    image.format = Some(office_oxide::ir::ImageFormat::Png);
                    self.current.elements.push(Element::Image(image));
                    self.flush(false)?;
                }
                Element::TextBox(value) => self.elements(&value.content, level)?,
                Element::Footnote(value) | Element::Endnote(value) => {
                    self.elements(&value.content, level)?
                }
                Element::PageBreak | Element::ColumnBreak => self.flush(false)?,
                Element::ThematicBreak => {
                    self.reserve(1)?;
                    self.current.elements.push(element.clone());
                    self.used += 1;
                }
                _ => return Err("Presentation does not support this document element".into()),
            }
        }
        Ok(())
    }

    fn list(&mut self, list: &List, level: u8) -> Result<(), String> {
        for (index, item) in list.items.iter().enumerate() {
            let marker = if list.ordered {
                format!(
                    "{}. ",
                    list.start_number
                        .unwrap_or(1)
                        .checked_add(
                            u32::try_from(index)
                                .map_err(|_| "Presentation list exceeds numbering range")?
                        )
                        .ok_or("Presentation list exceeds numbering range")?
                )
            } else {
                "• ".into()
            };
            let marker = format!("{}{marker}", "  ".repeat(usize::from(level)));
            let mut first = true;
            for element in &item.content {
                if let Element::Paragraph(value) = element {
                    self.paragraph(value, if first { Some(&marker) } else { None })?;
                    first = false;
                } else {
                    self.elements(std::slice::from_ref(element), level.saturating_add(1))?;
                }
            }
            if let Some(nested) = &item.nested {
                self.list(nested, level.saturating_add(1))?;
            }
        }
        Ok(())
    }

    fn code(&mut self, code: &CodeBlock) -> Result<(), String> {
        self.paragraph(
            &Paragraph {
                content: vec![InlineContent::Text(TextSpan {
                    text: code.content.clone(),
                    font_name: Some("Courier New".into()),
                    ..Default::default()
                })],
                ..Default::default()
            },
            None,
        )
    }

    pub fn reserve(&mut self, lines: usize) -> Result<(), String> {
        if self.used + lines > BODY_LINES {
            self.flush(false)?;
        }
        Ok(())
    }
}
