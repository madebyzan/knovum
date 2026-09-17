use serde::Deserialize;
use std::path::Path;

#[cfg(target_os = "windows")]
use std::process::Command;

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) enum ExternalLocation {
    Word {
        quote: Option<String>,
    },
    Slides {
        slide: usize,
    },
    Sheet {
        sheet: String,
        range: Option<String>,
    },
}

pub fn open_at_location(source: &Path, location: ExternalLocation) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return open_windows(source, location);

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (source, location);
        Err("Precise Microsoft Office navigation is currently available on Windows only".into())
    }
}

#[cfg(target_os = "windows")]
fn open_windows(source: &Path, location: ExternalLocation) -> Result<(), String> {
    let path = source.to_string_lossy().into_owned();
    let (script, envs) = match location {
        ExternalLocation::Word { quote } => (
            WORD_SCRIPT,
            vec![
                ("KNOVUM_FILE", path),
                ("KNOVUM_QUOTE", quote.unwrap_or_default()),
            ],
        ),
        ExternalLocation::Slides { slide } => (
            POWERPOINT_SCRIPT,
            vec![("KNOVUM_FILE", path), ("KNOVUM_SLIDE", slide.to_string())],
        ),
        ExternalLocation::Sheet { sheet, range } => (
            EXCEL_SCRIPT,
            vec![
                ("KNOVUM_FILE", path),
                ("KNOVUM_SHEET", sheet),
                ("KNOVUM_RANGE", range.unwrap_or_default()),
            ],
        ),
    };
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .envs(envs)
        .status()
        .map_err(|error| format!("Office automation could not start: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Office automation failed with status {status}"))
    }
}

#[cfg(target_os = "windows")]
const WORD_SCRIPT: &str = r#"
$ErrorActionPreference='Stop'
$word = New-Object -ComObject Word.Application
$word.Visible = $true
$doc = $word.Documents.Open($env:KNOVUM_FILE)
if ($env:KNOVUM_QUOTE) {
  $range = $doc.Content
  if ($range.Find.Execute($env:KNOVUM_QUOTE)) { $range.Select() }
}
$word.Activate()
"#;

#[cfg(target_os = "windows")]
const POWERPOINT_SCRIPT: &str = r#"
$ErrorActionPreference='Stop'
$app = New-Object -ComObject PowerPoint.Application
$app.Visible = $true
$presentation = $app.Presentations.Open($env:KNOVUM_FILE)
$slide = [int]$env:KNOVUM_SLIDE
if ($slide -lt 1 -or $slide -gt $presentation.Slides.Count) { throw 'Slide is outside the presentation' }
$app.ActiveWindow.View.GotoSlide($slide)
$app.Activate()
"#;

#[cfg(target_os = "windows")]
const EXCEL_SCRIPT: &str = r#"
$ErrorActionPreference='Stop'
$app = New-Object -ComObject Excel.Application
$app.Visible = $true
$book = $app.Workbooks.Open($env:KNOVUM_FILE)
$sheet = $book.Worksheets.Item($env:KNOVUM_SHEET)
$sheet.Activate()
if ($env:KNOVUM_RANGE) { $sheet.Range($env:KNOVUM_RANGE).Select() }
$app.ActiveWindow.Activate()
"#;
