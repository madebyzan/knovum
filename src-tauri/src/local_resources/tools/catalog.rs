use serde::Deserialize;
use std::{collections::HashMap, sync::OnceLock};

const MANIFEST: &str =
    include_str!("../../../resources/bundled-runtime/media-import-manifest.json");

#[derive(Clone, Copy)]
pub(crate) enum MediaImportToolKind {
    YtDlp,
    Deno,
}

#[derive(Clone)]
pub(crate) struct ToolDescriptor {
    pub(crate) kind: MediaImportToolKind,
    pub(crate) directory: &'static str,
    pub(crate) env_key: &'static str,
    pub(crate) version: String,
    pub(crate) asset: String,
    pub(crate) url: String,
    pub(crate) archive_sha256: String,
    pub(crate) binary_sha256: String,
    pub(crate) binary_name: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    yt_dlp: Release,
    deno: Release,
    targets: HashMap<String, Target>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Release {
    version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Target {
    yt_dlp: Asset,
    deno: Asset,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Asset {
    asset: String,
    sha256: String,
    #[serde(default)]
    binary_sha256: Option<String>,
}

pub(crate) fn descriptor(kind: MediaImportToolKind) -> Result<ToolDescriptor, String> {
    find_descriptor(kind)?
        .ok_or_else(|| "Media import tools are unavailable for this platform".to_string())
}

pub(super) fn find_descriptor(kind: MediaImportToolKind) -> Result<Option<ToolDescriptor>, String> {
    let manifest = manifest();
    let Some(target) = manifest.targets.get(super::platform_directory()) else {
        return Ok(None);
    };
    match kind {
        MediaImportToolKind::YtDlp => Ok(Some(ToolDescriptor {
            kind,
            directory: "yt-dlp",
            env_key: "KNOVUM_YT_DLP_PATH",
            version: manifest.yt_dlp.version.clone(),
            asset: target.yt_dlp.asset.clone(),
            url: format!(
                "https://github.com/yt-dlp/yt-dlp/releases/download/{}/{}",
                manifest.yt_dlp.version, target.yt_dlp.asset
            ),
            archive_sha256: target.yt_dlp.sha256.clone(),
            binary_sha256: target.yt_dlp.sha256.clone(),
            binary_name: super::yt_dlp_name(),
        })),
        MediaImportToolKind::Deno => Ok(Some(ToolDescriptor {
            kind,
            directory: "deno",
            env_key: "KNOVUM_DENO_PATH",
            version: manifest.deno.version.clone(),
            asset: target.deno.asset.clone(),
            url: format!(
                "https://github.com/denoland/deno/releases/download/v{}/{}",
                manifest.deno.version, target.deno.asset
            ),
            archive_sha256: target.deno.sha256.clone(),
            binary_sha256: target.deno.binary_sha256.clone().ok_or_else(|| {
                "Deno binary checksum is missing from the runtime manifest".to_string()
            })?,
            binary_name: super::deno_name(),
        })),
    }
}

fn manifest() -> &'static Manifest {
    static VALUE: OnceLock<Manifest> = OnceLock::new();
    VALUE.get_or_init(|| {
        serde_json::from_str(MANIFEST)
            .expect("repo-owned media import runtime manifest must be valid")
    })
}
