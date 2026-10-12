use super::DocumentSource;

pub(crate) fn render_source(input: &DocumentSource) -> Result<String, String> {
    super::source::render(
        &super::super::markdown::parse(input)?,
        &super::super::images::Images::load(input)?,
    )
}
