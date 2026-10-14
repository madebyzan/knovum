use super::super::{document::Citation, DocumentFormat};
use super::source;

fn render(input: &super::DocumentSource) -> Result<String, String> {
    let format: DocumentFormat =
        serde_json::from_str("\"html\"").map_err(|error| error.to_string())?;
    String::from_utf8(format.render(input)?).map_err(|error| error.to_string())
}

#[test]
fn standalone_html_preserves_document_structure_and_literal_content() {
    let mut input = source("# Report\n\n## Section\n\n**Bold** and *emphasis*, ~~strike~~ and `a < b`.\n\n> Quote\n\n7. First\n8. Second\n\n- [x] Done\n  - Nested\n\n| Name | Value |\n| :--- | ---: |\n| A | B |\n\n```html\n<script>alert('x')</script>\n```\n\n[Link](https://example.com/?a=1&b=2)\n\n---");
    input.title = "Report".into();
    input.citations.push(Citation {
        title: "<unsafe> & title".into(),
        quote: "literal <script> and **Markdown**".into(),
    });
    let html = render(&input).expect("render HTML");
    for expected in [
        "<!doctype html>",
        "charset=\"utf-8\"",
        "Content-Security-Policy",
        "<title>Report</title>",
        "<h2>Section</h2>",
        "<strong>Bold</strong>",
        "<em>emphasis</em>",
        "<del>strike</del>",
        "<code>a &lt; b</code>",
        "<blockquote>",
        "<ol start=\"7\">",
        "<ul>",
        "type=\"checkbox\" checked=\"\"",
        "<table>",
        "<thead>",
        "<tbody>",
        "style=\"text-align: right\"",
        "&lt;script&gt;alert('x')&lt;/script&gt;",
        "href=\"https://example.com/?a=1&amp;b=2\"",
        "<hr",
        "&lt;unsafe&gt; &amp; title",
        "literal &lt;script&gt; and **Markdown**",
        "</html>",
    ] {
        assert!(html.contains(expected), "missing {expected}");
    }
    assert_eq!(html.matches("<h1>Report</h1>").count(), 1);
    assert!(!html.contains("<script>"));
}

#[test]
fn html_rejects_active_content_private_links_and_unavailable_images() {
    for markdown in [
        "<script>alert(1)</script>",
        "[Run](javascript:alert)",
        "[Private](file:///private/report)",
        "[RootPath](/private/report)",
        "![Image](https://example.com/image.png)",
    ] {
        assert!(render(&source(markdown)).is_err(), "accepted {markdown}");
    }
}

#[test]
fn html_renders_relative_links_and_fragment_anchors() {
    let html =
        render(&source("[Relative](report.pdf) and [Anchor](#chapter-1)")).expect("render HTML");
    assert!(html.contains("href=\"report.pdf\""));
    assert!(html.contains("href=\"#chapter-1\""));
}

#[test]
fn html_escapes_title_and_attributes_and_keeps_unicode() {
    let mut input = source("中文 [Open](<https://example.com/\"quoted\">)");
    input.title = "\"<标题> & 'Title'".into();
    let html = render(&input).expect("render HTML");
    assert!(html.contains("\"&lt;标题&gt; &amp; 'Title'"));
    assert!(html.contains("href=\"https://example.com/%22quoted%22\""));
    assert!(html.contains("中文"));
}
