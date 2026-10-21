use std::io::Cursor;

use super::super::{
    input::{DocumentFileSource, DocumentInput},
    DocumentFormat,
};

fn pdf(contents: &str, image: bool) -> Vec<u8> {
    pdf_fixture(contents, image, false)
}

fn pdf_fixture(contents: &str, image: bool, blank_page: bool) -> Vec<u8> {
    let resource = if image {
        "/XObject << /Im0 5 0 R >>"
    } else {
        "/Font << /F1 5 0 R >>"
    };
    let object = if image {
        "<< /Type /XObject /Subtype /Image /Width 1000 /Height 1000 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /ASCIIHexDecode /Length 7 >>\nstream\n000000>\nendstream"
    } else {
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"
    };
    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".into(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] /Resources << {resource} >> /Contents 4 0 R >>"),
        format!("<< /Length {} >>\nstream\n{contents}\nendstream", contents.len()),
        object.into(),
    ];
    if blank_page {
        objects[1] = "<< /Type /Pages /Kids [3 0 R 6 0 R] /Count 2 >>".into();
        objects.push("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] /Resources << >> /Contents 7 0 R >>".into());
        objects.push("<< /Length 0 >>\nstream\n\nendstream".into());
    }
    let count = objects.len() + 1;
    let mut output = "%PDF-1.4\n".to_owned();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(output.len());
        output.push_str(&format!("{} 0 obj\n{object}\nendobj\n", index + 1));
    }
    let xref = output.len();
    output.push_str(&format!("xref\n0 {count}\n0000000000 65535 f \n"));
    for offset in offsets {
        output.push_str(&format!("{offset:010} 00000 n \n"));
    }
    output.push_str(&format!(
        "trailer\n<< /Size {count} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n"
    ));
    output.into_bytes()
}

fn convert(bytes: &[u8], format: DocumentFormat) -> Result<Vec<u8>, String> {
    let path = std::env::temp_dir().join(format!(
        "knovum-pdf-conversion-{}.pdf",
        uuid::Uuid::new_v4()
    ));
    std::fs::write(&path, bytes).unwrap();
    let input = DocumentInput::Pdf(super::super::input::PdfSource {
        file: DocumentFileSource {
            title: "Evidence".into(),
            source_revision: crate::file_inspection::content_revision(&path).unwrap(),
            path: path.clone(),
        },
    });
    let output = input.render(format);
    std::fs::remove_file(path).unwrap();
    output
}

#[test]
fn textual_pdf_converts_to_editable_word_content() {
    let bytes = convert(
        &pdf("BT /F1 12 Tf 72 720 Td (Verified content) Tj ET", false),
        DocumentFormat::Docx,
    )
    .unwrap();
    let output =
        office_oxide::Document::from_reader(Cursor::new(bytes), office_oxide::DocumentFormat::Docx)
            .unwrap();
    assert!(output.plain_text().contains("Verified content"));
    assert!(!output.plain_text().contains("Evidence"));
    let input = pdf("BT /F1 12 Tf 72 720 Td (Verified content) Tj ET", false);
    let text = String::from_utf8(convert(&input, DocumentFormat::Text).unwrap()).unwrap();
    assert!(text.contains("Verified content") && !text.contains("Evidence"));
    let html = String::from_utf8(convert(&input, DocumentFormat::Html).unwrap()).unwrap();
    assert!(html.contains("<title>Evidence</title>"));
    assert!(!html.contains("<h1>Evidence</h1>"));
}

#[test]
fn scanned_pdf_fails_instead_of_publishing_a_title_only_document() {
    // A scanned page has no text layer; plain text cannot carry its page image.
    let error = convert(
        &pdf("q 600 0 0 800 0 0 cm /Im0 Do Q", true),
        DocumentFormat::Text,
    )
    .unwrap_err();
    assert!(error.contains("no convertible content"), "{error}");
}

#[test]
fn a_blank_page_beside_text_still_converts() {
    let input = pdf_fixture(
        "BT /F1 12 Tf 72 720 Td (Verified content) Tj ET",
        false,
        true,
    );
    let text = String::from_utf8(convert(&input, DocumentFormat::Text).unwrap()).unwrap();
    assert!(text.contains("Verified content"));
}
