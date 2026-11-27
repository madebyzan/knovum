use super::symbols::{self, Symbol};

const MAX_DEPTH: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AccentKind {
    Hat,
    Bar,
    Tilde,
    Vec,
    Dot,
    DoubleDot,
    Overline,
    Underline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StyleKind {
    Roman,
    Bold,
    Italic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SpaceKind {
    Thin,
    Medium,
    Quad,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MatrixKind {
    Plain,
    Paren,
    Bracket,
    Pipe,
    Cases,
    Aligned,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum MathNode {
    Ident(String),
    Number(String),
    Operator(String),
    Symbol(Symbol),
    Text(String),
    Space(SpaceKind),
    Group(Vec<MathNode>),
    Frac(Vec<MathNode>, Vec<MathNode>),
    Binom(Vec<MathNode>, Vec<MathNode>),
    Sqrt {
        index: Option<Vec<MathNode>>,
        body: Vec<MathNode>,
    },
    Script {
        base: Box<MathNode>,
        sub: Option<Vec<MathNode>>,
        sup: Option<Vec<MathNode>>,
    },
    Accent {
        kind: AccentKind,
        body: Vec<MathNode>,
    },
    Styled {
        style: StyleKind,
        body: Vec<MathNode>,
    },
    Delim {
        left: String,
        right: String,
        body: Vec<MathNode>,
    },
    Matrix {
        kind: MatrixKind,
        rows: Vec<Vec<Vec<MathNode>>>,
    },
}

pub(super) fn parse(input: &str) -> Vec<MathNode> {
    let mut parser = Parser::new(input);
    parser.parse_sequence(None, 0)
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn starts_with(&self, needle: &str) -> bool {
        self.rest().starts_with(needle)
    }

    fn parse_sequence(&mut self, stop: Option<&str>, depth: usize) -> Vec<MathNode> {
        if depth > MAX_DEPTH {
            return vec![MathNode::Text(self.rest().to_owned())];
        }
        let mut nodes = Vec::new();
        while !self.rest().is_empty() {
            self.skip_whitespace();
            if self.rest().is_empty() {
                break;
            }
            if let Some(end) = stop {
                if self.starts_with(end) {
                    break;
                }
            }
            if stop == Some("}") && self.peek() == Some('}') {
                break;
            }
            if let Some(node) = self.parse_atom_with_scripts(depth) {
                nodes.push(node);
            }
        }
        nodes
    }

    fn parse_atom_with_scripts(&mut self, depth: usize) -> Option<MathNode> {
        let mut base = self.parse_atom(depth)?;
        let mut sub: Option<Vec<MathNode>> = None;
        let mut sup: Option<Vec<MathNode>> = None;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('_') if sub.is_none() => {
                    self.bump();
                    sub = Some(self.parse_arg(depth + 1));
                }
                Some('^') if sup.is_none() => {
                    self.bump();
                    sup = Some(self.parse_arg(depth + 1));
                }
                Some('\'') => {
                    self.bump();
                    let prime = MathNode::Operator("′".into());
                    sup.get_or_insert_with(Vec::new).push(prime);
                }
                _ => break,
            }
        }
        if sub.is_some() || sup.is_some() {
            base = MathNode::Script {
                base: Box::new(base),
                sub,
                sup,
            };
        }
        Some(base)
    }

    fn parse_atom(&mut self, depth: usize) -> Option<MathNode> {
        self.skip_whitespace();
        let ch = self.peek()?;
        match ch {
            '{' => Some(MathNode::Group(self.parse_braced(depth + 1))),
            '}' => {
                self.bump();
                None
            }
            '\\' => Some(self.parse_command(depth)),
            '0'..='9' => Some(MathNode::Number(self.read_number())),
            'a'..='z' | 'A'..='Z' => {
                self.bump();
                Some(MathNode::Ident(ch.to_string()))
            }
            '-' => {
                self.bump();
                Some(MathNode::Operator("−".into()))
            }
            '+' | '=' | '<' | '>' | '/' | ',' | ';' | ':' | '!' | '|' | '(' | ')' | '[' | ']'
            | '*' => {
                self.bump();
                Some(MathNode::Operator(ch.to_string()))
            }
            '_' | '^' => {
                self.bump();
                Some(MathNode::Operator(ch.to_string()))
            }
            _ => {
                self.bump();
                Some(MathNode::Text(ch.to_string()))
            }
        }
    }

    fn read_number(&mut self) -> String {
        let start = self.pos;
        let mut seen_dot = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.bump();
            } else if ch == '.' && !seen_dot {
                seen_dot = true;
                self.bump();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_owned()
    }

    fn parse_braced(&mut self, depth: usize) -> Vec<MathNode> {
        if self.peek() != Some('{') {
            return Vec::new();
        }
        self.bump();
        let content = self.parse_sequence(Some("}"), depth);
        if self.peek() == Some('}') {
            self.bump();
        }
        content
    }

    fn parse_arg(&mut self, depth: usize) -> Vec<MathNode> {
        self.skip_whitespace();
        if self.peek() == Some('{') {
            self.parse_braced(depth)
        } else if let Some(atom) = self.parse_atom(depth) {
            vec![atom]
        } else {
            Vec::new()
        }
    }

    fn parse_optional_bracket_arg(&mut self, depth: usize) -> Option<Vec<MathNode>> {
        self.skip_whitespace();
        if self.peek() != Some('[') {
            return None;
        }
        self.bump();
        let content = self.parse_sequence(Some("]"), depth);
        if self.peek() == Some(']') {
            self.bump();
        }
        Some(content)
    }

    fn parse_raw_braced_text(&mut self) -> String {
        self.skip_whitespace();
        if self.peek() != Some('{') {
            return self.bump().map_or_else(String::new, |ch| ch.to_string());
        }
        self.bump();
        let mut depth = 1usize;
        let start = self.pos;
        let mut end = start;
        while let Some(ch) = self.bump() {
            match ch {
                '\\' => {
                    let _ = self.bump();
                    end = self.pos;
                }
                '{' => {
                    depth += 1;
                    end = self.pos;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    end = self.pos;
                }
                _ => end = self.pos,
            }
        }
        self.input[start..end].to_owned()
    }

    fn read_command_name(&mut self) -> String {
        let start = self.pos;
        if let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                while self.peek().is_some_and(|c| c.is_ascii_alphabetic()) {
                    self.bump();
                }
            } else {
                self.bump();
            }
        }
        self.input[start..self.pos].to_owned()
    }

    fn parse_command(&mut self, depth: usize) -> MathNode {
        self.bump(); // consume '\'
        let name = self.read_command_name();
        match name.as_str() {
            "," | ":" | ";" | " " => MathNode::Space(SpaceKind::Thin),
            "!" => MathNode::Space(SpaceKind::Thin),
            "quad" => MathNode::Space(SpaceKind::Medium),
            "qquad" => MathNode::Space(SpaceKind::Quad),
            "%" | "$" | "&" | "#" | "_" | "{" | "}" => MathNode::Operator(name),
            "\\" => MathNode::Operator("; ".into()),
            "frac" | "dfrac" | "tfrac" | "cfrac" => {
                let num = self.parse_arg(depth + 1);
                let den = self.parse_arg(depth + 1);
                MathNode::Frac(num, den)
            }
            "binom" | "dbinom" | "tbinom" => {
                let top = self.parse_arg(depth + 1);
                let bottom = self.parse_arg(depth + 1);
                MathNode::Binom(top, bottom)
            }
            "sqrt" => {
                let index = self.parse_optional_bracket_arg(depth + 1);
                let body = self.parse_arg(depth + 1);
                MathNode::Sqrt { index, body }
            }
            "text" | "textrm" | "mbox" | "operatorname" => {
                MathNode::Text(self.parse_raw_braced_text())
            }
            "mathrm" | "operatorname*" => MathNode::Styled {
                style: StyleKind::Roman,
                body: self.parse_arg(depth + 1),
            },
            "mathbf" | "boldsymbol" | "bm" | "mathbb" => MathNode::Styled {
                style: StyleKind::Bold,
                body: self.parse_arg(depth + 1),
            },
            "mathit" | "mathcal" => MathNode::Styled {
                style: StyleKind::Italic,
                body: self.parse_arg(depth + 1),
            },
            "hat" | "widehat" => MathNode::Accent {
                kind: AccentKind::Hat,
                body: self.parse_arg(depth + 1),
            },
            "bar" => MathNode::Accent {
                kind: AccentKind::Bar,
                body: self.parse_arg(depth + 1),
            },
            "tilde" | "widetilde" => MathNode::Accent {
                kind: AccentKind::Tilde,
                body: self.parse_arg(depth + 1),
            },
            "vec" => MathNode::Accent {
                kind: AccentKind::Vec,
                body: self.parse_arg(depth + 1),
            },
            "dot" => MathNode::Accent {
                kind: AccentKind::Dot,
                body: self.parse_arg(depth + 1),
            },
            "ddot" => MathNode::Accent {
                kind: AccentKind::DoubleDot,
                body: self.parse_arg(depth + 1),
            },
            "overline" => MathNode::Accent {
                kind: AccentKind::Overline,
                body: self.parse_arg(depth + 1),
            },
            "underline" => MathNode::Accent {
                kind: AccentKind::Underline,
                body: self.parse_arg(depth + 1),
            },
            "left" => self.parse_delim(depth + 1),
            "right" => MathNode::Operator(self.read_delimiter()),
            "big" | "Big" | "bigg" | "Bigg" | "bigl" | "bigr" | "Bigl" | "Bigr" => {
                MathNode::Operator(self.read_delimiter())
            }
            "begin" => self.parse_environment(depth + 1),
            "displaystyle" | "textstyle" | "scriptstyle" | "limits" | "nolimits" => {
                MathNode::Group(Vec::new())
            }
            _ => {
                if let Some(symbol) = symbols::lookup(&name) {
                    MathNode::Symbol(symbol)
                } else {
                    MathNode::Text(format!("\\{name}"))
                }
            }
        }
    }

    fn read_delimiter(&mut self) -> String {
        self.skip_whitespace();
        match self.peek() {
            Some('.') => {
                self.bump();
                String::new()
            }
            Some('\\') => {
                self.bump();
                let name = self.read_command_name();
                match name.as_str() {
                    "{" | "}" | "|" => name,
                    _ => symbols::lookup(&name)
                        .map_or_else(|| name, |symbol| symbol.unicode.to_owned()),
                }
            }
            Some(ch) => {
                self.bump();
                ch.to_string()
            }
            None => String::new(),
        }
    }

    fn parse_delim(&mut self, depth: usize) -> MathNode {
        let left = self.read_delimiter();
        let body = self.parse_sequence(Some("\\right"), depth);
        let right = if self.starts_with("\\right") {
            self.pos += "\\right".len();
            self.read_delimiter()
        } else {
            String::new()
        };
        MathNode::Delim { left, right, body }
    }

    fn parse_environment(&mut self, depth: usize) -> MathNode {
        let env = self.parse_raw_braced_text();
        let end_marker = format!("\\end{{{env}}}");
        let kind = match env.as_str() {
            "pmatrix" => MatrixKind::Paren,
            "bmatrix" => MatrixKind::Bracket,
            "vmatrix" | "Vmatrix" => MatrixKind::Pipe,
            "cases" => MatrixKind::Cases,
            "aligned" | "align" | "align*" | "gathered" | "split" => MatrixKind::Aligned,
            _ => MatrixKind::Plain,
        };
        let raw_body = if let Some(found) = self.rest().find(&end_marker) {
            let body = &self.rest()[..found];
            self.pos += found + end_marker.len();
            body
        } else {
            let body = self.rest();
            self.pos = self.input.len();
            body
        };
        let rows = raw_body
            .split("\\\\")
            .map(str::trim)
            .filter(|row| !row.is_empty())
            .map(|row| {
                row.split('&')
                    .map(|cell| {
                        let mut sub = Parser::new(cell.trim());
                        sub.parse_sequence(None, depth)
                    })
                    .collect()
            })
            .collect();
        MathNode::Matrix { kind, rows }
    }
}
