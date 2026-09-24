use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};
use typst_layout::PagedDocument;

use crate::system_font::SYSTEM_SANS_FAMILY;

pub(crate) fn render(markdown: &str) -> Result<Vec<u8>, String> {
    let source = typst_source(markdown);
    let engine = TypstEngine::builder()
        .main_file(source)
        .search_fonts_with(TypstKitFontOptions::default())
        .build();
    let compilation = engine.compile();
    if !compilation.warnings.is_empty() {
        return Err(format!(
            "PDF typesetting warnings: {:?}",
            compilation.warnings
        ));
    }
    let document: PagedDocument = compilation
        .output
        .map_err(|errors| format!("PDF typesetting failed: {errors:?}"))?;
    typst_pdf::pdf(&document, &Default::default())
        .map_err(|errors| format!("PDF export failed: {errors:?}"))
}

fn typst_source(markdown: &str) -> String {
    let mut output = format!("#set page(paper: \"a4\", margin: (x: 24mm, y: 22mm))\n#set text(font: \"{SYSTEM_SANS_FAMILY}\", size: 10.5pt, lang: \"zh\")\n#set par(justify: true, leading: 0.72em)\n#set heading(numbering: none)\n\n");
    let mut lists = Vec::<Option<u64>>::new();
    let mut code_block = false;
    for event in Parser::new_ext(markdown, Options::all()) {
        write_event(&mut output, &mut lists, &mut code_block, event);
    }
    output
}

fn write_event(
    output: &mut String,
    lists: &mut Vec<Option<u64>>,
    code_block: &mut bool,
    event: Event<'_>,
) {
    match event {
        Event::Start(Tag::Heading { level, .. }) => {
            output.push('\n');
            output.push_str(heading_marker(level));
            output.push(' ');
        }
        Event::Start(Tag::Strong) => output.push('*'),
        Event::Start(Tag::Emphasis) => output.push('_'),
        Event::Start(Tag::Link { dest_url, .. }) => {
            output.push_str("#link(\"");
            output.push_str(&escape_string(&dest_url));
            output.push_str("\")[");
        }
        Event::Start(Tag::List(start)) => lists.push(start),
        Event::Start(Tag::Item) => write_item_prefix(output, lists),
        Event::Start(Tag::CodeBlock(_)) => {
            *code_block = true;
            output.push_str("\n#raw(\"");
        }
        Event::Text(text) if *code_block => output.push_str(&escape_string(&text)),
        Event::Text(text) => output.push_str(&escape_markup(&text)),
        Event::Code(text) => {
            output.push_str("#raw(\"");
            output.push_str(&escape_string(&text));
            output.push_str("\")");
        }
        Event::SoftBreak => output.push(' '),
        Event::HardBreak => output.push_str(" \\\n"),
        Event::Rule => output.push_str("\n#line(length: 100%)\n"),
        Event::End(TagEnd::Strong) => output.push('*'),
        Event::End(TagEnd::Emphasis) => output.push('_'),
        Event::End(TagEnd::Link) => output.push(']'),
        Event::End(TagEnd::Heading(_)) | Event::End(TagEnd::Paragraph) => output.push_str("\n\n"),
        Event::End(TagEnd::Item) => output.push('\n'),
        Event::End(TagEnd::List(_)) => {
            lists.pop();
            output.push('\n');
        }
        Event::End(TagEnd::CodeBlock) => {
            *code_block = false;
            output.push_str("\", block: true)\n\n");
        }
        _ => {}
    }
}

fn write_item_prefix(output: &mut String, lists: &mut [Option<u64>]) {
    output.push_str(&"  ".repeat(lists.len().saturating_sub(1)));
    if let Some(Some(index)) = lists.last_mut() {
        output.push_str(&format!("{index}. "));
        *index += 1;
    } else {
        output.push_str("- ");
    }
}

fn heading_marker(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "=",
        HeadingLevel::H2 => "==",
        HeadingLevel::H3 => "===",
        HeadingLevel::H4 => "====",
        HeadingLevel::H5 => "=====",
        HeadingLevel::H6 => "======",
    }
}

fn escape_markup(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| {
            if matches!(character, '\\' | '#' | '*' | '_' | '$' | '@' | '<' | '>') {
                vec!['\\', character]
            } else {
                vec![character]
            }
        })
        .collect()
}

fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
pub(crate) fn source_for_test(markdown: &str) -> String {
    typst_source(markdown)
}
