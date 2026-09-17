use office_oxide::ir::{Element, Section};

pub(super) struct Content<'a> {
    pub part: &'static str,
    pub elements: &'a [Element],
}

/// Borrow all logical content exposed by the library; never duplicate its image-bearing IR.
pub(super) fn parts(section: &Section) -> impl Iterator<Item = Content<'_>> {
    [
        ("body", Some(section.elements.as_slice())),
        (
            "header",
            section
                .header
                .as_ref()
                .map(|value| value.content.as_slice()),
        ),
        (
            "footer",
            section
                .footer
                .as_ref()
                .map(|value| value.content.as_slice()),
        ),
        (
            "first_page_header",
            section
                .first_page_header
                .as_ref()
                .map(|value| value.content.as_slice()),
        ),
        (
            "first_page_footer",
            section
                .first_page_footer
                .as_ref()
                .map(|value| value.content.as_slice()),
        ),
        (
            "even_page_header",
            section
                .even_page_header
                .as_ref()
                .map(|value| value.content.as_slice()),
        ),
        (
            "even_page_footer",
            section
                .even_page_footer
                .as_ref()
                .map(|value| value.content.as_slice()),
        ),
    ]
    .into_iter()
    .filter_map(|(part, elements)| elements.map(|elements| Content { part, elements }))
    .filter(|content| !content.elements.is_empty())
}
