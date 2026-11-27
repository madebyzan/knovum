use super::{
    parser::{AccentKind, MathNode, MatrixKind, SpaceKind, StyleKind},
    symbols::SymbolKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MathScript {
    Subscript,
    Superscript,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MathRun {
    pub text: String,
    pub italic: bool,
    pub bold: bool,
    pub script: Option<MathScript>,
}

pub(super) fn to_mathml(nodes: &[MathNode], display: bool) -> String {
    let mode = if display { "block" } else { "inline" };
    let mut out =
        format!("<math xmlns=\"http://www.w3.org/1998/Math/MathML\" display=\"{mode}\"><mrow>");
    write_mathml_seq(&mut out, nodes);
    out.push_str("</mrow></math>");
    out
}

fn write_mathml_seq(out: &mut String, nodes: &[MathNode]) {
    for node in nodes {
        write_mathml_node(out, node);
    }
}

fn write_mathml_group(out: &mut String, nodes: &[MathNode]) {
    if nodes.len() == 1 {
        write_mathml_node(out, &nodes[0]);
    } else {
        out.push_str("<mrow>");
        write_mathml_seq(out, nodes);
        out.push_str("</mrow>");
    }
}

fn write_mathml_node(out: &mut String, node: &MathNode) {
    match node {
        MathNode::Ident(value) => {
            out.push_str("<mi>");
            push_xml(out, value);
            out.push_str("</mi>");
        }
        MathNode::Number(value) => {
            out.push_str("<mn>");
            push_xml(out, value);
            out.push_str("</mn>");
        }
        MathNode::Operator(value) => {
            out.push_str("<mo>");
            push_xml(out, value);
            out.push_str("</mo>");
        }
        MathNode::Symbol(symbol) => match symbol.kind {
            SymbolKind::Ident => {
                out.push_str("<mi>");
                push_xml(out, symbol.unicode);
                out.push_str("</mi>");
            }
            SymbolKind::Operator => {
                out.push_str("<mo>");
                push_xml(out, symbol.unicode);
                out.push_str("</mo>");
            }
            SymbolKind::Function => {
                out.push_str("<mi mathvariant=\"normal\">");
                push_xml(out, symbol.unicode);
                out.push_str("</mi>");
            }
        },
        MathNode::Text(value) => {
            out.push_str("<mtext>");
            push_xml(out, value);
            out.push_str("</mtext>");
        }
        MathNode::Space(kind) => {
            let width = match kind {
                SpaceKind::Thin => "0.167em",
                SpaceKind::Medium => "0.5em",
                SpaceKind::Quad => "1em",
            };
            out.push_str(&format!("<mspace width=\"{width}\"/>"));
        }
        MathNode::Group(children) => write_mathml_group(out, children),
        MathNode::Frac(num, den) => {
            out.push_str("<mfrac>");
            write_mathml_group(out, num);
            write_mathml_group(out, den);
            out.push_str("</mfrac>");
        }
        MathNode::Binom(top, bottom) => {
            out.push_str("<mrow><mo>(</mo><mfrac linethickness=\"0\">");
            write_mathml_group(out, top);
            write_mathml_group(out, bottom);
            out.push_str("</mfrac><mo>)</mo></mrow>");
        }
        MathNode::Sqrt { index, body } => match index {
            Some(idx) => {
                out.push_str("<mroot>");
                write_mathml_group(out, body);
                write_mathml_group(out, idx);
                out.push_str("</mroot>");
            }
            None => {
                out.push_str("<msqrt>");
                write_mathml_seq(out, body);
                out.push_str("</msqrt>");
            }
        },
        MathNode::Script { base, sub, sup } => match (sub, sup) {
            (Some(sub), Some(sup)) => {
                out.push_str("<msubsup>");
                write_mathml_node(out, base);
                write_mathml_group(out, sub);
                write_mathml_group(out, sup);
                out.push_str("</msubsup>");
            }
            (Some(sub), None) => {
                out.push_str("<msub>");
                write_mathml_node(out, base);
                write_mathml_group(out, sub);
                out.push_str("</msub>");
            }
            (None, Some(sup)) => {
                out.push_str("<msup>");
                write_mathml_node(out, base);
                write_mathml_group(out, sup);
                out.push_str("</msup>");
            }
            (None, None) => write_mathml_node(out, base),
        },
        MathNode::Accent { kind, body } => {
            let (tag, mark) = match kind {
                AccentKind::Hat => ("mover", "^"),
                AccentKind::Bar | AccentKind::Overline => ("mover", "¯"),
                AccentKind::Tilde => ("mover", "~"),
                AccentKind::Vec => ("mover", "→"),
                AccentKind::Dot => ("mover", "˙"),
                AccentKind::DoubleDot => ("mover", "¨"),
                AccentKind::Underline => ("munder", "_"),
            };
            out.push_str(&format!("<{tag} accent=\"true\">"));
            write_mathml_group(out, body);
            out.push_str("<mo>");
            push_xml(out, mark);
            out.push_str(&format!("</mo></{tag}>"));
        }
        MathNode::Styled { style, body } => {
            let variant = match style {
                StyleKind::Roman => "normal",
                StyleKind::Bold => "bold",
                StyleKind::Italic => "italic",
            };
            out.push_str(&format!("<mstyle mathvariant=\"{variant}\">"));
            write_mathml_seq(out, body);
            out.push_str("</mstyle>");
        }
        MathNode::Delim { left, right, body } => {
            out.push_str("<mrow>");
            if !left.is_empty() {
                out.push_str("<mo stretchy=\"true\">");
                push_xml(out, left);
                out.push_str("</mo>");
            }
            write_mathml_seq(out, body);
            if !right.is_empty() {
                out.push_str("<mo stretchy=\"true\">");
                push_xml(out, right);
                out.push_str("</mo>");
            }
            out.push_str("</mrow>");
        }
        MathNode::Matrix { kind, rows } => {
            let (left, right) = matrix_delims(*kind);
            out.push_str("<mrow>");
            if !left.is_empty() {
                out.push_str(&format!("<mo stretchy=\"true\">{left}</mo>"));
            }
            out.push_str("<mtable>");
            for row in rows {
                out.push_str("<mtr>");
                for cell in row {
                    out.push_str("<mtd>");
                    write_mathml_seq(out, cell);
                    out.push_str("</mtd>");
                }
                out.push_str("</mtr>");
            }
            out.push_str("</mtable>");
            if !right.is_empty() {
                out.push_str(&format!("<mo stretchy=\"true\">{right}</mo>"));
            }
            out.push_str("</mrow>");
        }
    }
}

/// Emits injection-safe Typst math markup built exclusively from the parsed AST.
/// All user text and unknown commands are emitted as quoted strings, never raw Typst code.
pub(super) fn to_typst(nodes: &[MathNode], display: bool) -> String {
    let mut inner = String::new();
    write_typst_seq(&mut inner, nodes);
    if inner.trim().is_empty() {
        return String::new();
    }
    if display {
        format!("$ {inner} $")
    } else {
        format!("${inner}$")
    }
}

fn write_typst_seq(out: &mut String, nodes: &[MathNode]) {
    for (index, node) in nodes.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        write_typst_node(out, node);
    }
}

fn write_typst_node(out: &mut String, node: &MathNode) {
    match node {
        MathNode::Ident(value) => {
            if value.len() == 1 && value.chars().all(|c| c.is_ascii_alphabetic()) {
                out.push_str(value);
            } else {
                out.push_str(&typst_quoted(value));
            }
        }
        MathNode::Number(value) => out.push_str(value),
        MathNode::Operator(value) => match value.as_str() {
            "−" | "-" => out.push('-'),
            "+" | "=" | "<" | ">" | "," | ";" | ":" | "!" | "*" => out.push_str(value),
            "|" => out.push_str("bar.v"),
            "(" => out.push_str("paren.l"),
            ")" => out.push_str("paren.r"),
            "[" => out.push_str("bracket.l"),
            "]" => out.push_str("bracket.r"),
            "/" => out.push_str("slash"),
            "′" => out.push('\''),
            "{" => out.push_str("brace.l"),
            "}" => out.push_str("brace.r"),
            _ => out.push_str(&typst_quoted(value)),
        },
        MathNode::Symbol(symbol) => out.push_str(symbol.typst),
        MathNode::Text(value) => out.push_str(&typst_quoted(value)),
        MathNode::Space(kind) => out.push_str(match kind {
            SpaceKind::Thin => "thin",
            SpaceKind::Medium => "quad",
            SpaceKind::Quad => "wide",
        }),
        MathNode::Group(children) => {
            out.push('(');
            write_typst_seq(out, children);
            out.push(')');
        }
        MathNode::Frac(num, den) => {
            out.push_str("frac(");
            write_typst_seq(out, num);
            out.push_str(", ");
            write_typst_seq(out, den);
            out.push(')');
        }
        MathNode::Binom(top, bottom) => {
            out.push_str("binom(");
            write_typst_seq(out, top);
            out.push_str(", ");
            write_typst_seq(out, bottom);
            out.push(')');
        }
        MathNode::Sqrt { index, body } => match index {
            Some(idx) => {
                out.push_str("root(");
                write_typst_seq(out, idx);
                out.push_str(", ");
                write_typst_seq(out, body);
                out.push(')');
            }
            None => {
                out.push_str("sqrt(");
                write_typst_seq(out, body);
                out.push(')');
            }
        },
        MathNode::Script { base, sub, sup } => {
            write_typst_node(out, base);
            if let Some(sub) = sub {
                out.push_str("_(");
                write_typst_seq(out, sub);
                out.push(')');
            }
            if let Some(sup) = sup {
                out.push_str("^(");
                write_typst_seq(out, sup);
                out.push(')');
            }
        }
        MathNode::Accent { kind, body } => {
            let func = match kind {
                AccentKind::Hat => "hat",
                AccentKind::Bar => "macron",
                AccentKind::Tilde => "tilde",
                AccentKind::Vec => "arrow",
                AccentKind::Dot => "dot",
                AccentKind::DoubleDot => "dot.double",
                AccentKind::Overline => "overline",
                AccentKind::Underline => "underline",
            };
            out.push_str(func);
            out.push('(');
            write_typst_seq(out, body);
            out.push(')');
        }
        MathNode::Styled { style, body } => {
            let func = match style {
                StyleKind::Roman => "upright",
                StyleKind::Bold => "bold",
                StyleKind::Italic => "italic",
            };
            out.push_str(func);
            out.push('(');
            write_typst_seq(out, body);
            out.push(')');
        }
        MathNode::Delim { left, right, body } => {
            out.push_str("lr(");
            if !left.is_empty() {
                out.push_str(&typst_delim(left));
                out.push(' ');
            }
            write_typst_seq(out, body);
            if !right.is_empty() {
                out.push(' ');
                out.push_str(&typst_delim(right));
            }
            out.push(')');
        }
        MathNode::Matrix { kind, rows } => {
            let func = match kind {
                MatrixKind::Cases => "cases(",
                MatrixKind::Aligned => "grid(columns: auto, ",
                MatrixKind::Paren => "mat(delim: \"(\", ",
                MatrixKind::Bracket => "mat(delim: \"[\", ",
                MatrixKind::Pipe => "mat(delim: \"|\", ",
                MatrixKind::Plain => "mat(delim: none, ",
            };
            out.push_str(func);
            for (row_idx, row) in rows.iter().enumerate() {
                if row_idx > 0 {
                    out.push_str(if matches!(kind, MatrixKind::Cases) {
                        ", "
                    } else {
                        "; "
                    });
                }
                for (col_idx, cell) in row.iter().enumerate() {
                    if col_idx > 0 {
                        out.push_str(if matches!(kind, MatrixKind::Cases) {
                            " & "
                        } else {
                            ", "
                        });
                    }
                    write_typst_seq(out, cell);
                }
            }
            out.push(')');
        }
    }
}

pub(super) fn to_runs(nodes: &[MathNode]) -> Vec<MathRun> {
    let mut runs = Vec::new();
    collect_runs(nodes, false, false, None, &mut runs);
    coalesce_runs(runs)
}

pub(super) fn to_plain(nodes: &[MathNode]) -> String {
    let runs = to_runs(nodes);
    let mut out = String::new();
    for run in runs {
        match run.script {
            Some(MathScript::Superscript) => push_unicode_script(&mut out, &run.text, true),
            Some(MathScript::Subscript) => push_unicode_script(&mut out, &run.text, false),
            None => out.push_str(&run.text),
        }
    }
    out
}

fn collect_runs(
    nodes: &[MathNode],
    bold: bool,
    force_roman: bool,
    script: Option<MathScript>,
    out: &mut Vec<MathRun>,
) {
    for node in nodes {
        match node {
            MathNode::Ident(value) => push_run(out, value, !force_roman, bold, script),
            MathNode::Number(value) => push_run(out, value, false, bold, script),
            MathNode::Operator(value) => {
                let spaced = matches!(value.as_str(), "+" | "−" | "=" | "<" | ">" | "|");
                if spaced && script.is_none() {
                    push_run(out, &format!(" {value} "), false, bold, script);
                } else {
                    push_run(out, value, false, bold, script);
                }
            }
            MathNode::Symbol(symbol) => {
                let italic = matches!(symbol.kind, SymbolKind::Ident) && !force_roman;
                let text = if matches!(symbol.kind, SymbolKind::Operator) && script.is_none() {
                    format!(" {} ", symbol.unicode)
                } else {
                    symbol.unicode.to_owned()
                };
                push_run(out, &text, italic, bold, script);
            }
            MathNode::Text(value) => push_run(out, value, false, bold, script),
            MathNode::Space(_) => push_run(out, " ", false, bold, script),
            MathNode::Group(children) => collect_runs(children, bold, force_roman, script, out),
            MathNode::Frac(num, den) => {
                let wrap_num = needs_parens(num);
                let wrap_den = needs_parens(den);
                if wrap_num {
                    push_run(out, "(", false, bold, script);
                }
                collect_runs(num, bold, force_roman, script, out);
                if wrap_num {
                    push_run(out, ")", false, bold, script);
                }
                push_run(out, " / ", false, bold, script);
                if wrap_den {
                    push_run(out, "(", false, bold, script);
                }
                collect_runs(den, bold, force_roman, script, out);
                if wrap_den {
                    push_run(out, ")", false, bold, script);
                }
            }
            MathNode::Binom(top, bottom) => {
                push_run(out, "C(", false, bold, script);
                collect_runs(top, bold, force_roman, script, out);
                push_run(out, ", ", false, bold, script);
                collect_runs(bottom, bold, force_roman, script, out);
                push_run(out, ")", false, bold, script);
            }
            MathNode::Sqrt { index, body } => {
                if let Some(idx) = index {
                    collect_runs(idx, bold, force_roman, Some(MathScript::Superscript), out);
                }
                push_run(out, "√(", false, bold, script);
                collect_runs(body, bold, force_roman, script, out);
                push_run(out, ")", false, bold, script);
            }
            MathNode::Script { base, sub, sup } => {
                collect_runs(
                    std::slice::from_ref(base.as_ref()),
                    bold,
                    force_roman,
                    script,
                    out,
                );
                if let Some(sub) = sub {
                    collect_runs(
                        sub,
                        bold,
                        force_roman,
                        script.or(Some(MathScript::Subscript)),
                        out,
                    );
                }
                if let Some(sup) = sup {
                    collect_runs(
                        sup,
                        bold,
                        force_roman,
                        script.or(Some(MathScript::Superscript)),
                        out,
                    );
                }
            }
            MathNode::Accent { kind, body } => {
                collect_runs(body, bold, force_roman, script, out);
                let combining = match kind {
                    AccentKind::Hat => "\u{0302}",
                    AccentKind::Bar | AccentKind::Overline => "\u{0304}",
                    AccentKind::Tilde => "\u{0303}",
                    AccentKind::Vec => "\u{20d7}",
                    AccentKind::Dot => "\u{0307}",
                    AccentKind::DoubleDot => "\u{0308}",
                    AccentKind::Underline => "\u{0332}",
                };
                push_run(out, combining, false, bold, script);
            }
            MathNode::Styled { style, body } => match style {
                StyleKind::Roman => collect_runs(body, bold, true, script, out),
                StyleKind::Bold => collect_runs(body, true, true, script, out),
                StyleKind::Italic => collect_runs(body, bold, false, script, out),
            },
            MathNode::Delim { left, right, body } => {
                if !left.is_empty() {
                    push_run(out, left, false, bold, script);
                }
                collect_runs(body, bold, force_roman, script, out);
                if !right.is_empty() {
                    push_run(out, right, false, bold, script);
                }
            }
            MathNode::Matrix { kind, rows } => {
                let (left, right) = matrix_delims(*kind);
                if !left.is_empty() {
                    push_run(out, left, false, bold, script);
                }
                for (row_idx, row) in rows.iter().enumerate() {
                    if row_idx > 0 {
                        push_run(out, "; ", false, bold, script);
                    }
                    for (col_idx, cell) in row.iter().enumerate() {
                        if col_idx > 0 {
                            push_run(out, ", ", false, bold, script);
                        }
                        collect_runs(cell, bold, force_roman, script, out);
                    }
                }
                if !right.is_empty() {
                    push_run(out, right, false, bold, script);
                }
            }
        }
    }
}

fn push_run(
    out: &mut Vec<MathRun>,
    text: &str,
    italic: bool,
    bold: bool,
    script: Option<MathScript>,
) {
    if text.is_empty() {
        return;
    }
    out.push(MathRun {
        text: text.to_owned(),
        italic,
        bold,
        script,
    });
}

fn coalesce_runs(runs: Vec<MathRun>) -> Vec<MathRun> {
    let mut merged: Vec<MathRun> = Vec::with_capacity(runs.len());
    for run in runs {
        if let Some(last) = merged.last_mut() {
            if last.italic == run.italic && last.bold == run.bold && last.script == run.script {
                last.text.push_str(&run.text);
                continue;
            }
        }
        merged.push(run);
    }
    for run in &mut merged {
        while run.text.contains("  ") {
            run.text = run.text.replace("  ", " ");
        }
    }
    if let Some(first) = merged.first_mut() {
        let trimmed = first.text.trim_start().to_owned();
        first.text = trimmed;
    }
    if let Some(last) = merged.last_mut() {
        let trimmed = last.text.trim_end().to_owned();
        last.text = trimmed;
    }
    merged
        .into_iter()
        .filter(|run| !run.text.is_empty())
        .collect()
}

fn needs_parens(nodes: &[MathNode]) -> bool {
    nodes.len() > 1
}

fn matrix_delims(kind: MatrixKind) -> (&'static str, &'static str) {
    match kind {
        MatrixKind::Plain | MatrixKind::Aligned => ("", ""),
        MatrixKind::Paren => ("(", ")"),
        MatrixKind::Bracket => ("[", "]"),
        MatrixKind::Pipe => ("|", "|"),
        MatrixKind::Cases => ("{", ""),
    }
}

fn push_xml(out: &mut String, value: &str) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
}

fn typst_quoted(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

fn typst_delim(value: &str) -> String {
    match value {
        "(" | ")" | "[" | "]" | "|" => value.to_owned(),
        "{" => "brace.l".into(),
        "}" => "brace.r".into(),
        "⟨" => "chevron.l".into(),
        "⟩" => "chevron.r".into(),
        "⌊" => "floor.l".into(),
        "⌋" => "floor.r".into(),
        "⌈" => "ceil.l".into(),
        "⌉" => "ceil.r".into(),
        "‖" => "bar.v.double".into(),
        _ => typst_quoted(value),
    }
}

fn push_unicode_script(out: &mut String, text: &str, sup: bool) {
    let mapped: Option<String> = text
        .chars()
        .map(|ch| if sup { to_sup(ch) } else { to_sub(ch) })
        .collect();
    if let Some(pretty) = mapped {
        out.push_str(&pretty);
    } else {
        out.push(if sup { '^' } else { '_' });
        if text.chars().count() > 1 {
            out.push('(');
            out.push_str(text);
            out.push(')');
        } else {
            out.push_str(text);
        }
    }
}

fn to_sup(ch: char) -> Option<char> {
    Some(match ch {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        '+' => '⁺',
        '-' | '−' => '⁻',
        '=' => '⁼',
        '(' => '⁽',
        ')' => '⁾',
        'n' => 'ⁿ',
        'i' => 'ⁱ',
        _ => return None,
    })
}

fn to_sub(ch: char) -> Option<char> {
    Some(match ch {
        '0' => '₀',
        '1' => '₁',
        '2' => '₂',
        '3' => '₃',
        '4' => '₄',
        '5' => '₅',
        '6' => '₆',
        '7' => '₇',
        '8' => '₈',
        '9' => '₉',
        '+' => '₊',
        '-' | '−' => '₋',
        '=' => '₌',
        '(' => '₍',
        ')' => '₎',
        'a' => 'ₐ',
        'e' => 'ₑ',
        'h' => 'ₕ',
        'i' => 'ᵢ',
        'j' => 'ⱼ',
        'k' => 'ₖ',
        'l' => 'ₗ',
        'm' => 'ₘ',
        'n' => 'ₙ',
        'o' => 'ₒ',
        'p' => 'ₚ',
        'r' => 'ᵣ',
        's' => 'ₛ',
        't' => 'ₜ',
        'u' => 'ᵤ',
        'v' => 'ᵥ',
        'x' => 'ₓ',
        _ => return None,
    })
}
