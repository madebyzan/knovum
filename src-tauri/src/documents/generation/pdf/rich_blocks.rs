use office_oxide::ir::{Element, Paragraph, ParagraphAlignment};

use super::{rich::Writer, source::string};

impl Writer {
    pub(super) fn block(&mut self, element: &Element) -> Result<(), String> {
        match element {
            Element::Paragraph(value) => self.paragraph(value)?,
            Element::Heading(value) => {
                self.source
                    .push_str(&format!("#heading(level: {})[", value.level));
                self.inlines(&value.content)?;
                self.source.push(']');
            }
            Element::CodeBlock(value) => self.source.push_str(&format!(
                "#raw({}, block: true, lang: {})",
                string(&value.content),
                string(value.language.as_deref().unwrap_or(""))
            )),
            Element::Image(value) => self.image(value)?,
            Element::Table(value) => self.table(value)?,
            Element::List(value) => {
                if value.ordered {
                    self.source.push_str(&format!(
                        "#enum(start: {}, tight: false,",
                        value.start_number.unwrap_or(1)
                    ));
                } else {
                    self.source.push_str("#list(tight: false,");
                }
                for item in &value.items {
                    self.source.push_str("[\n");
                    self.blocks(&item.content)?;
                    if let Some(nested) = &item.nested {
                        self.block(&Element::List(nested.clone()))?;
                    }
                    self.source.push_str("],\n");
                }
                self.source.push(')');
            }
            Element::TextBox(value) => {
                self.source
                    .push_str("#block(inset: (left: 12pt), stroke: (left: 1pt + luma(180)))[\n");
                self.blocks(&value.content)?;
                self.source.push(']');
            }
            Element::Footnote(value) | Element::Endnote(value) => {
                self.source.push_str("#footnote[");
                self.blocks(&value.content)?;
                self.source.push(']');
            }
            Element::PageBreak => self.source.push_str("#pagebreak()"),
            Element::ColumnBreak => self.source.push_str("#colbreak()"),
            Element::ThematicBreak => self.source.push_str("#line(length: 100%)"),
            _ => return Err("Unsupported document element in PDF renderer".into()),
        }
        Ok(())
    }

    fn paragraph(&mut self, value: &Paragraph) -> Result<(), String> {
        if value.page_break_before {
            self.source.push_str("#pagebreak()\n");
        }
        let alignment = match value.alignment {
            Some(ParagraphAlignment::Center) => "center",
            Some(ParagraphAlignment::Right) => "right",
            _ => "left",
        };
        self.source.push_str(&format!(
            "#align({alignment})[#par(justify: {})[",
            matches!(value.alignment, Some(ParagraphAlignment::Justify))
        ));
        self.inlines(&value.content)?;
        self.source.push_str("]]");
        Ok(())
    }
}
