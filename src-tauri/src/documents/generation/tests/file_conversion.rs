use std::io::Cursor;

use office_oxide::{
    docx::write::DocxWriter,
    ir::{Image, ImageFormat},
    Document,
};

use super::super::{
    input::{DocumentFileSource, DocumentInput},
    DocumentFormat,
};

struct DocumentFileFixture(DocumentFileSource);

impl DocumentFileFixture {
    fn new(extension: &str, bytes: &[u8]) -> Self {
        let path = std::env::temp_dir().join(format!(
            "knovum-conversion-{}.{}",
            uuid::Uuid::new_v4(),
            extension
        ));
        std::fs::write(&path, bytes).unwrap();
        Self(DocumentFileSource {
            title: "Report <&>".into(),
            source_revision: crate::file_inspection::content_revision(&path).unwrap(),
            path,
        })
    }

    fn render(&self, format: DocumentFormat) -> Result<Vec<u8>, String> {
        DocumentInput::File(self.0.clone()).render(format)
    }
}

impl Drop for DocumentFileFixture {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0.path).unwrap();
    }
}

fn illustrated_word() -> Vec<u8> {
    let mut writer = DocxWriter::new();
    writer.add_heading("Verified heading", 1);
    writer.add_paragraph("Literal <script> & content");
    writer.add_table(&[vec!["Name", "Value"], vec!["Evidence", "42"]]);
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 2)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    writer.add_ir_image(&Image {
        data: Some(png.into_inner()),
        format: Some(ImageFormat::Png),
        alt_text: Some("Figure <&>".into()),
        ..Default::default()
    });
    let mut bytes = Cursor::new(Vec::new());
    writer.write_to(&mut bytes).unwrap();
    bytes.into_inner()
}

#[test]
fn office_html_preserves_tables_and_embeds_images_without_active_markup() {
    let fixture = DocumentFileFixture::new("docx", &illustrated_word());
    let output = String::from_utf8(fixture.render(DocumentFormat::Html).unwrap()).unwrap();
    assert!(output.contains("<table"));
    assert!(output.contains("Evidence") && output.contains("42"));
    assert!(output.contains("data:image/png;base64,"));
    assert!(output.contains("Literal &lt;script&gt; &amp; content"));
    assert!(output.contains("<title>Report &lt;&amp;&gt;</title>"));
    assert!(!output.contains("<script>"));
    assert!(fixture
        .render(DocumentFormat::Pdf)
        .unwrap_err()
        .contains("original-layout PDF conversion is not supported"));
}

#[test]
fn office_word_conversion_keeps_real_tables_and_embedded_image_bytes() {
    let word = Document::from_reader(
        Cursor::new(illustrated_word()),
        office_oxide::DocumentFormat::Docx,
    )
    .unwrap();
    let mut slides = Cursor::new(Vec::new());
    office_oxide::create::create_from_ir_to_writer(
        &word.to_ir(),
        office_oxide::DocumentFormat::Pptx,
        &mut slides,
    )
    .unwrap();
    let fixture = DocumentFileFixture::new("pptx", &slides.into_inner());
    let output = fixture.render(DocumentFormat::Docx).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(output)).unwrap();
    assert!(archive
        .file_names()
        .any(|name| name.starts_with("word/media/")));
    let mut body = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("word/document.xml").unwrap(),
        &mut body,
    )
    .unwrap();
    assert!(body.contains("<w:tbl") && body.contains("Evidence"));
}

#[test]
fn spreadsheet_conversion_preserves_worksheet_and_cell_content() {
    use office_oxide::xlsx::write::{CellData, XlsxWriter};
    let mut writer = XlsxWriter::new();
    writer.add_sheet("Revenue").add_row(vec![
        CellData::String("Amount".into()),
        CellData::Number(42.0),
    ]);
    let mut bytes = Cursor::new(Vec::new());
    writer.write_to(&mut bytes).unwrap();
    let fixture = DocumentFileFixture::new("xlsx", &bytes.into_inner());
    let output = String::from_utf8(fixture.render(DocumentFormat::Text).unwrap()).unwrap();
    assert!(output.contains("Revenue") && output.contains("Amount") && output.contains("42"));
    assert!(!output.contains(&fixture.0.title));
    assert!(fixture.render(DocumentFormat::Pptx).is_err());
}

#[test]
fn conversion_rejects_stale_oversized_or_unsupported_source_files() {
    let fixture = DocumentFileFixture::new("docx", &illustrated_word());
    std::fs::write(&fixture.0.path, b"changed").unwrap();
    assert!(fixture
        .render(DocumentFormat::Text)
        .unwrap_err()
        .contains("changed"));
    std::fs::OpenOptions::new()
        .write(true)
        .open(&fixture.0.path)
        .unwrap()
        .set_len(128 * 1024 * 1024 + 1)
        .unwrap();
    assert!(fixture
        .render(DocumentFormat::Text)
        .unwrap_err()
        .contains("128 MiB"));
    let unsupported = DocumentFileFixture::new("bin", b"opaque");
    assert!(unsupported
        .render(DocumentFormat::Text)
        .unwrap_err()
        .contains("Unsupported"));
}

#[test]
fn office_conversion_rejects_private_file_hyperlinks() {
    let mut writer = DocxWriter::new();
    let mut run = office_oxide::docx::write::Run::new("Private link");
    run.hyperlink = Some("file:///private/document".into());
    writer.add_ir_paragraph(&[run], None);
    let mut bytes = Cursor::new(Vec::new());
    writer.write_to(&mut bytes).unwrap();
    let bytes = bytes.into_inner();
    let mut archive = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    let mut relationships = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("word/_rels/document.xml.rels").unwrap(),
        &mut relationships,
    )
    .unwrap();
    assert!(relationships.contains("file:///private/document"));
    let fixture = DocumentFileFixture::new("docx", &bytes);
    assert!(fixture
        .render(DocumentFormat::Html)
        .unwrap_err()
        .contains("hyperlinks"));
}
