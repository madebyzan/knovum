use pulldown_cmark::{html::push_html, Event, HeadingLevel, Parser, Tag, TagEnd};

use super::document::DocumentSource;

pub(crate) fn render(source: &DocumentSource, metadata_title: &str) -> Result<Vec<u8>, String> {
    // Keep the export subset, budgets and title policy shared with the other formats.
    let document = super::markdown::parse(source)?;
    let mut body = Parser::new_ext(&source.markdown, super::markdown::OPTIONS)
        .map(validate_event)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter();
    if document.leading_title_removed {
        for event in body.by_ref() {
            if matches!(event, Event::End(TagEnd::Heading(HeadingLevel::H1))) {
                break;
            }
        }
    }
    let mut output = String::new();
    if !document.title.is_empty() {
        push_html(
            &mut output,
            [
                build_heading_event(HeadingLevel::H1),
                Event::Text(document.title.as_str().into()),
                Event::End(TagEnd::Heading(HeadingLevel::H1)),
            ]
            .into_iter(),
        );
    }
    push_html(&mut output, body);
    append_citations(&mut output, source);
    Ok(build_document(metadata_title.trim(), &output))
}

/// Only engine-generated, validated HTML fragments enter this document shell.
pub(super) fn build_document(title: &str, body: &str) -> Vec<u8> {
    let mut escaped_title = String::new();
    push_html(&mut escaped_title, [Event::Text(title.into())].into_iter());
    format!(
        "<!doctype html>\n<html lang=\"und\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; img-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'\">\
         <title>{escaped_title}</title><style>{}</style></head><body><main>{body}</main></body></html>\n",
        include_str!("document.css"),
    ).into_bytes()
}

fn validate_event(event: Event<'_>) -> Result<Event<'_>, String> {
    match &event {
        Event::Start(Tag::Link { dest_url, .. }) => super::links::validate(dest_url)?,
        Event::Start(Tag::Image { .. }) => {
            return Err("HTML export does not support images".into());
        }
        _ => {}
    }
    Ok(event)
}

fn build_heading_event(level: HeadingLevel) -> Event<'static> {
    Event::Start(Tag::Heading {
        level,
        id: None,
        classes: vec![],
        attrs: vec![],
    })
}

fn append_citations(output: &mut String, source: &DocumentSource) {
    if source.citations.is_empty() {
        return;
    }
    push_html(
        output,
        [
            build_heading_event(HeadingLevel::H2),
            Event::Text("Sources".into()),
            Event::End(TagEnd::Heading(HeadingLevel::H2)),
        ]
        .into_iter(),
    );
    for (index, citation) in source.citations.iter().enumerate() {
        push_html(
            output,
            [
                Event::Start(Tag::Paragraph),
                Event::Text(format!("[C{}] ", index + 1).into()),
                Event::Start(Tag::Strong),
                Event::Text(citation.title.as_str().into()),
                Event::End(TagEnd::Strong),
                Event::Text(format!(" — {}", citation.quote).into()),
                Event::End(TagEnd::Paragraph),
            ]
            .into_iter(),
        );
    }
}
