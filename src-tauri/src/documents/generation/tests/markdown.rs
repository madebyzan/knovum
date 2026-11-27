use super::super::{
    document::{Block, Citation, Inline},
    markdown, text,
};
use super::{fixture, source};

#[test]
fn title_is_retained_without_duplicating_a_matching_leading_heading() {
    let mut input = source("# **Report**\n\nBody");
    input.title = "Report".into();
    let document = markdown::parse(&input).unwrap();
    assert_eq!(document.title, "Report");
    assert_eq!(
        document.blocks,
        vec![Block::Paragraph(vec![Inline::Text("Body".into())])]
    );
}

#[test]
fn a_distinct_body_heading_is_not_removed() {
    let mut input = source("# Introduction");
    input.title = "Report".into();
    let document = markdown::parse(&input).unwrap();
    assert!(matches!(
        document.blocks[0],
        Block::Heading { level: 1, .. }
    ));
}

#[test]
fn citations_are_literal_structured_content() {
    let mut input = source("Body");
    input.citations.push(Citation {
        title: "[title] **not markup**".into(),
        quote: "*quote*\n# not a heading".into(),
    });
    let document = markdown::parse(&input).unwrap();
    let Block::Paragraph(content) = document.blocks.last().unwrap() else {
        panic!("expected a source paragraph");
    };
    assert_eq!(content.len(), 3);
    assert_eq!(
        text::inline_text(content, true),
        "[C1] [title] **not markup** — *quote*\n# not a heading"
    );
}

#[test]
fn tight_lists_preserve_nested_block_structure() {
    let document = markdown::parse(&source("- outer\n  - inner\n- sibling")).unwrap();
    let Block::List { start: None, items } = &document.blocks[0] else {
        panic!("expected a bullet list");
    };
    assert_eq!(items.len(), 2);
    assert!(matches!(items[0][0], Block::Paragraph(_)));
    assert!(matches!(items[0][1], Block::List { .. }));
}

#[test]
fn common_fixture_preserves_tables_code_and_ordered_start() {
    let document = markdown::parse(&fixture()).unwrap();
    assert!(document.blocks.iter().any(|b| matches!(b, Block::Table(_))));
    assert!(document
        .blocks
        .iter()
        .any(|b| matches!(b, Block::List { start: Some(7), .. })));
    assert!(document
        .blocks
        .iter()
        .any(|b| matches!(b, Block::Code { language, .. } if language == "rust")));
}

/// Markdown carries HTML all the time - badge rows, centred headers, `<sub>` captions - and a
/// document is the content that markup wraps.
#[test]
fn html_markup_contributes_the_content_it_wraps() {
    let markdown = concat!(
        "<div align=\"center\">\n",
        "<b>Knovum</b>&nbsp;&#8212; one&amp;all\n",
        "<br>\n",
        "<img src=\"https://img.example/badge.svg\" alt=\"build passing\">\n",
        "</div>"
    );
    let document = markdown::parse(&source(markdown)).unwrap();
    let Block::Paragraph(content) = &document.blocks[0] else {
        panic!("expected the wrapped content as a paragraph");
    };
    assert_eq!(
        text::inline_text(content, true),
        "Knovum\u{a0}— one&all \nbuild passing",
        "text and line breaks are content; a no-break space is not spacing"
    );
}

/// A script or a stylesheet is markup whose text is not prose, so none of it reaches the document.
#[test]
fn markup_without_prose_contributes_nothing() {
    for markdown in [
        "<script>if (a < b) danger()</script>",
        "<style>.a { color: red }</style>",
        "<!-- a remark -->",
    ] {
        assert!(
            markdown::parse(&source(markdown))
                .unwrap()
                .blocks
                .is_empty(),
            "{markdown} says nothing about the document"
        );
    }
}

#[test]
fn input_size_and_nesting_are_bounded() {
    assert!(markdown::parse(&source(&"x".repeat(4 * 1024 * 1024 + 1))).is_err());
    assert!(markdown::parse(&source(&format!("{}text", "> ".repeat(100)))).is_err());
}

#[test]
fn invalid_xml_characters_are_rejected_before_rendering() {
    for value in ["hello\0", "hello\u{ffff}", "hello\u{fffe}"] {
        assert!(markdown::parse(&source(value)).is_err());
    }
}

#[test]
fn inline_and_block_math_render_across_all_export_formats() {
    use super::super::DocumentFormat;
    let md = "收益率为 $R_t = \\frac{P_t - P_{t-1}}{P_{t-1}}$，能量 $E = mc^2$。\n\n$$\n\\sigma = \\sqrt{\\frac{1}{N}\\sum_{i=1}^N (x_i - \\mu)^2}\n$$\n\n:::gallery{layout=\"gallery\"}\n\n&nbsp;\n\n:::";
    let input = source(md);

    let html = String::from_utf8(DocumentFormat::Html.render(&input).unwrap()).unwrap();
    assert!(html.contains("<math "));
    assert!(html.contains("<mfrac>"));
    assert!(html.contains("<msqrt>"));
    assert!(html.contains("σ") && html.contains("∑"));
    assert!(!html.contains(":::gallery"));
    assert!(!html.contains("&nbsp;"));

    let txt = String::from_utf8(DocumentFormat::Text.render(&input).unwrap()).unwrap();
    assert!(txt.contains("E = mc²"));
    assert!(txt.contains("σ = √("));
    assert!(!txt.contains(":::gallery"));

    let pdf_source = super::super::pdf::tests::render_source(&input).unwrap();
    assert!(pdf_source.contains("frac("));
    assert!(pdf_source.contains("sqrt("));
    assert!(!pdf_source.contains(":::gallery"));

    let pdf = DocumentFormat::Pdf.render(&input).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));

    let docx = DocumentFormat::Docx.render(&input).unwrap();
    assert!(docx.starts_with(b"PK"));

    let pptx = DocumentFormat::Pptx.render(&input).unwrap();
    assert!(pptx.starts_with(b"PK"));
}

#[test]
fn typst_math_quotes_untrusted_commands_without_code_execution() {
    let input = source("Check $#read(\"private\") + \\evil{x}$");
    let rendered = super::super::pdf::tests::render_source(&input).unwrap();
    assert!(rendered.contains("\"#\""));
    assert!(rendered.contains("\"\\\\evil\""));
}

#[test]
fn callout_mermaid_and_cjk_inline_math_render_across_all_formats() {
    use std::io::Cursor;

    use super::super::{
        document::{DocumentSource, RenderedImage},
        images::diagram_destination,
        DocumentFormat,
    };

    let diagram = "flowchart TD\n  A[\"全球宏观\"] --> B[\"中国宏观\"]";
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(64, 32)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();

    let md = format!(
        "# 中国股市研究\n\n> [!NOTE]\n> 本文梳理研究路径。\n\n```mermaid\n{diagram}\n```\n\n\
         | 指标 | 公式 |\n| --- | --- |\n| 市盈率 | $PE = \\frac{{P}}{{EPS}}$ |\n\n\
         - $SMB$：规模因子；\n- $\\alpha_i$：超额收益；\n- 动量因子 $MOM$；\n- 区间 $[0, 1)$ 与 $P(A|B)$。\n"
    );
    let input = DocumentSource {
        title: "中国股市研究".into(),
        markdown: md,
        citations: vec![],
        images: vec![],
        rendered_images: vec![RenderedImage {
            destination: diagram_destination(diagram),
            bytes: png.into_inner(),
        }],
    };

    let html = String::from_utf8(DocumentFormat::Html.render(&input).unwrap()).unwrap();
    assert!(html.contains("callout-note") && !html.contains("[!NOTE]"));
    assert!(html.contains("data:image/png;base64,") && html.contains("Mermaid diagram"));
    assert!(html.contains("<math ") && html.contains("α") && html.contains("<mfrac>"));

    let docx = DocumentFormat::Docx.render(&input).unwrap();
    let docx_zip = zip::ZipArchive::new(Cursor::new(docx)).unwrap();
    assert!(docx_zip
        .file_names()
        .any(|name| name.unwrap().starts_with("word/media/")));

    let pptx = DocumentFormat::Pptx.render(&input).unwrap();
    let pptx_zip = zip::ZipArchive::new(Cursor::new(pptx)).unwrap();
    assert!(pptx_zip
        .file_names()
        .any(|name| name.unwrap().starts_with("ppt/media/")));

    let pdf_source = super::super::pdf::tests::render_source(&input).unwrap();
    assert!(
        pdf_source.contains("#align(center)[#image(\"image-0.png\""),
        "PDF source must emit rendered Mermaid diagrams as standalone centered image blocks"
    );
    let pdf = DocumentFormat::Pdf.render(&input).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    assert!(
        pdf.windows(b"/Image".len())
            .any(|window| window == b"/Image"),
        "PDF output must embed rendered Mermaid diagram image XObjects"
    );
    // Verify embedded CJK font glyphs actually landed in the PDF (a Latin-only fallback PDF is tiny).
    assert!(
        pdf.len() > 8_000,
        "PDF output too small ({} bytes); CJK font may be missing",
        pdf.len()
    );
}
