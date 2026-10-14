use office_oxide::ir::{DocumentIR, Element, Image};

pub(super) struct Writer {
    pub source: String,
    pub binaries: Vec<(String, Vec<u8>)>,
    image_bytes: usize,
}

pub(super) fn render(ir: &DocumentIR) -> Result<Vec<u8>, String> {
    let mut writer = Writer {
        source: include_str!("document.typ").into(),
        binaries: vec![],
        image_bytes: 0,
    };
    writer.source.push_str(&format!(
        "#set text(font: {}, size: 10.5pt)\n",
        super::source::string(crate::desktop::system_font::SYSTEM_SANS_FAMILY)
    ));
    for section in &ir.sections {
        writer.blocks(&section.elements)?;
    }
    super::compile(writer.source, writer.binaries)
}

impl Writer {
    pub fn blocks(&mut self, elements: &[Element]) -> Result<(), String> {
        for element in elements {
            self.block(element)?;
            self.source.push_str("\n\n");
        }
        Ok(())
    }

    pub fn image(&mut self, image: &Image) -> Result<(), String> {
        let image_data = super::super::images::decode(
            image
                .data
                .as_deref()
                .ok_or("Document image bytes are missing")?,
        )?;
        self.image_bytes += image_data.bytes.len();
        if self.image_bytes > 64 * 1024 * 1024 {
            return Err("Document images exceed 64 MiB".into());
        }
        let filename = format!("image-{}.png", self.binaries.len());
        let scale = (468.0 / f64::from(image_data.width))
            .min(595.0 / f64::from(image_data.height))
            .min(0.75);
        self.source.push_str(&format!(
            "#image({}, width: {}pt, alt: {})",
            super::source::string(&filename),
            f64::from(image_data.width) * scale,
            super::source::string(image.alt_text.as_deref().unwrap_or(""))
        ));
        self.binaries.push((filename, image_data.bytes));
        Ok(())
    }
}
