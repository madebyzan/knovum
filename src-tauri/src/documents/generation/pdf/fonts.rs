use std::path::{Path, PathBuf};

use fontdb::Database;

pub(super) struct VerifiedFonts {
    pub files: Vec<Vec<u8>>,
    pub sans_families: Vec<String>,
    pub mono_families: Vec<String>,
    pub math_families: Vec<String>,
    pub has_cjk: bool,
}

/// Loads a focused set of public platform UI, CJK, monospace, and math fonts and verifies every
/// candidate file through `fontdb` so unparseable/reserved system files are never accepted.
pub(super) fn load_verified_fonts() -> VerifiedFonts {
    let mut files = Vec::new();
    let mut sans_families = Vec::new();
    let mut mono_families = Vec::new();
    let mut math_families = Vec::new();
    let mut has_cjk = false;

    for (role, candidates) in platform_font_groups() {
        for path in candidates {
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let families = inspect_font_families(&bytes);
            if families.is_empty() {
                continue;
            }
            match role {
                FontRole::CjkSans => {
                    has_cjk = true;
                    extend_unique(&mut sans_families, families);
                }
                FontRole::LatinSans => extend_unique(&mut sans_families, families),
                FontRole::Mono => extend_unique(&mut mono_families, families),
                FontRole::Math => extend_unique(&mut math_families, families),
            }
            files.push(bytes);
            break;
        }
    }

    VerifiedFonts {
        files,
        sans_families,
        mono_families,
        math_families,
        has_cjk,
    }
}

#[derive(Clone, Copy)]
enum FontRole {
    CjkSans,
    LatinSans,
    Mono,
    Math,
}

fn inspect_font_families(bytes: &[u8]) -> Vec<String> {
    let mut db = Database::new();
    db.load_font_data(bytes.to_vec());
    let mut families = Vec::new();
    for face in db.faces() {
        for (name, _) in &face.families {
            if !name.is_empty() && !name.starts_with('.') && !families.contains(name) {
                families.push(name.clone());
            }
        }
    }
    families
}

fn extend_unique(target: &mut Vec<String>, items: Vec<String>) {
    for item in items {
        if !target.contains(&item) {
            target.push(item);
        }
    }
}

#[cfg(target_os = "macos")]
fn platform_font_groups() -> Vec<(FontRole, Vec<PathBuf>)> {
    vec![
        (
            FontRole::CjkSans,
            vec![
                PathBuf::from("/System/Library/Fonts/PingFang.ttc"),
                PathBuf::from("/System/Library/Fonts/Hiragino Sans GB.ttc"),
                PathBuf::from("/System/Library/Fonts/STHeiti Medium.ttc"),
                PathBuf::from("/System/Library/Fonts/STHeiti Light.ttc"),
            ],
        ),
        (
            FontRole::LatinSans,
            vec![
                PathBuf::from("/System/Library/Fonts/Helvetica.ttc"),
                PathBuf::from("/System/Library/Fonts/HelveticaNeue.ttc"),
            ],
        ),
        (
            FontRole::Mono,
            vec![
                PathBuf::from("/System/Library/Fonts/Menlo.ttc"),
                PathBuf::from("/System/Library/Fonts/Monaco.ttf"),
            ],
        ),
        (
            FontRole::Math,
            vec![
                PathBuf::from("/System/Library/Fonts/Supplemental/STIXTwoMath.otf"),
                PathBuf::from("/System/Library/Fonts/Apple Symbols.ttf"),
            ],
        ),
    ]
}

#[cfg(target_os = "windows")]
fn platform_font_groups() -> Vec<(FontRole, Vec<PathBuf>)> {
    let root = std::env::var_os("WINDIR")
        .or_else(|| std::env::var_os("SYSTEMROOT"))
        .map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from)
        .join("Fonts");
    let font = |name: &str| root.join(name);
    vec![
        (FontRole::CjkSans, vec![font("msyh.ttc")]),
        (FontRole::CjkSans, vec![font("msyhbd.ttc")]),
        (FontRole::LatinSans, vec![font("segoeui.ttf")]),
        (FontRole::LatinSans, vec![font("segoeuib.ttf")]),
        (FontRole::Mono, vec![font("consola.ttf")]),
        (FontRole::Math, vec![font("cambria.ttc")]),
    ]
}

#[cfg(target_os = "linux")]
fn platform_font_groups() -> Vec<(FontRole, Vec<PathBuf>)> {
    vec![
        (
            FontRole::CjkSans,
            vec![
                PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
                PathBuf::from("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc"),
                PathBuf::from("/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc"),
            ],
        ),
        (
            FontRole::CjkSans,
            vec![
                PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc"),
                PathBuf::from("/usr/share/fonts/noto-cjk/NotoSansCJK-Bold.ttc"),
                PathBuf::from("/usr/share/fonts/google-noto-cjk/NotoSansCJK-Bold.ttc"),
            ],
        ),
        (
            FontRole::LatinSans,
            vec![
                PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
                PathBuf::from("/usr/share/fonts/TTF/DejaVuSans.ttf"),
            ],
        ),
        (
            FontRole::Mono,
            vec![
                PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"),
                PathBuf::from("/usr/share/fonts/TTF/DejaVuSansMono.ttf"),
            ],
        ),
        (
            FontRole::Math,
            vec![
                PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuMathTeXGyre.ttf"),
                PathBuf::from("/usr/share/fonts/opentype/stix/STIXTwoMath-Regular.otf"),
            ],
        ),
    ]
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn platform_font_groups() -> Vec<(FontRole, Vec<PathBuf>)> {
    Vec::new()
}

pub(super) fn uses_custom_font(source: &str, bundle: &VerifiedFonts) -> bool {
    let mut rest = source;
    while let Some(pos) = rest.find("font: \"") {
        let after = &rest[pos + "font: \"".len()..];
        let Some(end) = after.find('"') else {
            break;
        };
        let family = &after[..end];
        if !is_known_family(family, bundle) {
            return true;
        }
        rest = &after[end + 1..];
    }
    false
}

fn is_known_family(family: &str, bundle: &VerifiedFonts) -> bool {
    let lower = family.trim().to_ascii_lowercase();
    if matches!(lower.as_str(), "monospace" | "courier new") || Path::new(family).is_absolute() {
        return true;
    }
    bundle
        .sans_families
        .iter()
        .chain(&bundle.mono_families)
        .chain(&bundle.math_families)
        .any(|known| known.to_ascii_lowercase() == lower)
}

pub(super) fn typst_font_preamble(bundle: &VerifiedFonts) -> String {
    let mut sans = vec![crate::desktop::system_font::SYSTEM_SANS_FAMILY.to_owned()];
    extend_unique(&mut sans, bundle.sans_families.clone());
    extend_unique(
        &mut sans,
        vec![
            "Hiragino Sans GB".into(),
            "Heiti SC".into(),
            "Helvetica".into(),
            "Segoe UI".into(),
            "DejaVu Sans".into(),
        ],
    );

    let mut mono = bundle.mono_families.clone();
    extend_unique(
        &mut mono,
        vec![
            "Menlo".into(),
            "Consolas".into(),
            "DejaVu Sans Mono".into(),
            "Courier New".into(),
        ],
    );

    let mut math = bundle.math_families.clone();
    extend_unique(
        &mut math,
        vec![
            "STIX Two Math".into(),
            "Cambria Math".into(),
            "New Computer Modern Math".into(),
        ],
    );

    format!(
        "#set text(font: ({}), size: 10.5pt)\n\
         #show raw: set text(font: ({}))\n\
         #show math.equation: set text(font: ({}))\n",
        format_family_list(&sans),
        format_family_list(&mono),
        format_family_list(&math),
    )
}

fn format_family_list(families: &[String]) -> String {
    families
        .iter()
        .map(|name| super::source::string(name))
        .collect::<Vec<_>>()
        .join(", ")
}
