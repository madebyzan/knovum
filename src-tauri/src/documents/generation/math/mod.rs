mod emit;
mod parser;
mod symbols;

use office_oxide::ir::{InlineContent, TextSpan, VerticalAlign};

pub(crate) use emit::{MathRun, MathScript};

pub(crate) fn to_mathml(latex: &str, display: bool) -> String {
    let nodes = parser::parse(latex.trim());
    emit::to_mathml(&nodes, display)
}

pub(crate) fn to_typst(latex: &str, display: bool) -> String {
    let nodes = parser::parse(latex.trim());
    emit::to_typst(&nodes, display)
}

pub(crate) fn to_runs(latex: &str) -> Vec<MathRun> {
    let nodes = parser::parse(latex.trim());
    emit::to_runs(&nodes)
}

pub(crate) fn to_plain(latex: &str) -> String {
    let nodes = parser::parse(latex.trim());
    emit::to_plain(&nodes)
}

pub(crate) fn to_ir_spans(latex: &str, base: &TextSpan) -> Vec<InlineContent> {
    to_runs(latex)
        .into_iter()
        .map(|run| {
            let mut span = base.clone();
            span.text = run.text;
            span.italic = span.italic || run.italic;
            span.bold = span.bold || run.bold;
            if let Some(script) = run.script {
                span.vertical_align = Some(match script {
                    MathScript::Subscript => VerticalAlign::Subscript,
                    MathScript::Superscript => VerticalAlign::Superscript,
                });
            }
            InlineContent::Text(span)
        })
        .collect()
}
