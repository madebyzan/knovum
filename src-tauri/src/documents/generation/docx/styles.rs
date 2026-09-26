use docx_rs::{Docx, RunFonts, Style, StyleType};

use crate::system_font::SYSTEM_SANS_FAMILY;

pub(super) fn document() -> Docx {
    let normal = Style::new("KnovumBody", StyleType::Paragraph)
        .name("Knovum Body")
        .size(21)
        .fonts(fonts("Arial"));
    let mut doc = Docx::new()
        .add_style(normal)
        .add_style(
            Style::new("Title", StyleType::Paragraph)
                .name("Title")
                .based_on("KnovumBody")
                .size(40)
                .bold(),
        )
        .add_style(
            Style::new("KnovumCode", StyleType::Character)
                .name("Knovum Code")
                .fonts(fonts("Courier New"))
                .size(18),
        )
        .add_style(
            Style::new("KnovumCodeBlock", StyleType::Paragraph)
                .name("Knovum Code Block")
                .based_on("KnovumBody"),
        );
    for (index, size) in [32, 28, 25, 23, 22, 21].into_iter().enumerate() {
        doc = doc.add_style(
            Style::new(format!("Heading{}", index + 1), StyleType::Paragraph)
                .name(format!("heading {}", index + 1))
                .based_on("KnovumBody")
                .next("KnovumBody")
                .outline_lvl(index)
                .size(size)
                .bold(),
        );
    }
    doc
}

fn fonts(latin: &str) -> RunFonts {
    RunFonts::new()
        .ascii(latin)
        .hi_ansi(latin)
        .east_asia(SYSTEM_SANS_FAMILY)
}
