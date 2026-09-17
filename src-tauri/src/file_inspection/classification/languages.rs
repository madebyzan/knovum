use std::path::Path;

use super::{extension, file_name_lower};

pub(super) fn source_language(path: &Path) -> Option<&'static str> {
    file_name_lower(path)
        .as_deref()
        .and_then(exact_language)
        .or_else(|| extension_language(extension(path).as_deref()))
}

fn exact_language(name: &str) -> Option<&'static str> {
    match name {
        "dockerfile" => Some("dockerfile"),
        "makefile" | "gnumakefile" => Some("makefile"),
        "cargo.lock" => Some("toml"),
        "cmakelists.txt" => Some("cmake"),
        "gemfile" | "podfile" => Some("ruby"),
        "go.mod" | "go.sum" => Some("go"),
        "yarn.lock" => Some("text"),
        ".editorconfig" => Some("ini"),
        ".gitignore" | ".gitattributes" | ".prettierrc" | ".eslintrc" => Some("text"),
        _ => None,
    }
}

fn extension_language(extension: Option<&str>) -> Option<&'static str> {
    match extension {
        Some("js" | "mjs" | "cjs") => Some("javascript"),
        Some("ts" | "mts" | "cts") => Some("typescript"),
        Some("jsx") => Some("jsx"),
        Some("tsx") => Some("tsx"),
        Some("html" | "htm") => Some("html"),
        Some("css" | "scss" | "sass" | "less") => Some("css"),
        Some("vue") => Some("vue"),
        Some("svelte") => Some("svelte"),
        Some("astro") => Some("astro"),
        Some("json" | "jsonc" | "json5") => Some("json"),
        Some("yaml" | "yml") => Some("yaml"),
        Some("toml") => Some("toml"),
        Some("xml") => Some("xml"),
        Some("rs") => Some("rust"),
        Some("py" | "pyi") => Some("python"),
        Some("go") => Some("go"),
        Some("java") => Some("java"),
        Some("kt" | "kts") => Some("kotlin"),
        Some("swift") => Some("swift"),
        Some("c" | "h") => Some("c"),
        Some("cc" | "cpp" | "cxx" | "hpp" | "hh") => Some("cpp"),
        Some("cs") => Some("csharp"),
        Some("php") => Some("php"),
        Some("rb") => Some("ruby"),
        Some("sh" | "bash" | "zsh" | "fish") => Some("shell"),
        Some("bat" | "cmd") => Some("batch"),
        Some("ps1") => Some("powershell"),
        Some("sql") => Some("sql"),
        Some("graphql" | "gql") => Some("graphql"),
        Some("ini" | "conf" | "cfg" | "properties") => Some("ini"),
        Some("diff" | "patch") => Some("diff"),
        Some("tex") => Some("latex"),
        Some("mdx") => Some("mdx"),
        Some("gradle") => Some("gradle"),
        _ => None,
    }
}
