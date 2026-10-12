use office_oxide::ir::{InlineContent, TextSpan, VerticalAlign};

use super::{rich::Writer, source::string};

impl Writer {
    pub(super) fn inlines(&mut self, content: &[InlineContent]) -> Result<(), String> {
        for inline in content {
            match inline {
                InlineContent::Text(value) => self.span(value)?,
                InlineContent::LineBreak => self.source.push_str("#linebreak()"),
                _ => return Err("Unsupported document reference in PDF renderer".into()),
            }
        }
        Ok(())
    }

    fn span(&mut self, span: &TextSpan) -> Result<(), String> {
        let mut wrappers = Vec::new();
        if let Some(link) = &span.hyperlink {
            super::super::links::validate(link)?;
            self.source.push_str(&format!("#link({})[", string(link)));
            wrappers.push(());
        }
        for (active, name) in [
            (span.bold, "strong"),
            (span.italic, "emph"),
            (span.strikethrough, "strike"),
            (span.underline.is_some(), "underline"),
        ] {
            if active {
                self.source.push_str(&format!("#{name}["));
                wrappers.push(());
            }
        }
        if let Some(alignment) = &span.vertical_align {
            let name = match alignment {
                VerticalAlign::Superscript => "super",
                VerticalAlign::Subscript => "sub",
                _ => return Err("Unsupported vertical text alignment".into()),
            };
            self.source.push_str(&format!("#{name}["));
            wrappers.push(());
        }
        if let Some(rgb) = span.highlight {
            self.source.push_str(&format!(
                "#highlight(fill: rgb({}, {}, {}))[",
                rgb[0], rgb[1], rgb[2]
            ));
            wrappers.push(());
        }
        self.source.push_str("#text(");
        if let Some(size) = span.font_size_half_pt {
            self.source
                .push_str(&format!("size: {}pt,", f64::from(size) / 2.0));
        }
        if let Some(font) = &span.font_name {
            self.source.push_str(&format!("font: {},", string(font)));
        }
        if let Some(rgb) = span.color {
            self.source
                .push_str(&format!("fill: rgb({}, {}, {}),", rgb[0], rgb[1], rgb[2]));
        }
        self.source.push_str(&string(&span.text));
        self.source.push(')');
        for _ in wrappers {
            self.source.push(']');
        }
        Ok(())
    }
}
