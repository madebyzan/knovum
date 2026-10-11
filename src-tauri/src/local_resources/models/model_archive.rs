use super::catalog::{self, ImageModelFile, ModelKind};
use crate::local_resources::{
    download,
    preparation::{
        contracts::{ResourcePreparationProgress, ResourcePreparationStage},
        RESOURCE_PREPARATION_CANCELLED,
    },
    verification,
};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy)]
pub(crate) struct ModelArchive {
    pub url: &'static str,
    pub sha256: &'static str,
    pub bytes: u64,
    pub directory: &'static str,
    pub members: &'static [ImageModelFile],
}

impl ModelArchive {
    pub fn owns(self, kind: ModelKind) -> bool {
        matches!(kind, ModelKind::ImagePart(part) if self.members.contains(&part.file))
    }
    pub async fn download(
        self,
        snapshot: super::snapshots::SnapshotKind,
        directory: &Path,
        progress: &mut impl FnMut(ResourcePreparationProgress),
        cancellation: &CancellationToken,
    ) -> Result<(), String> {
        let path = directory.join("release.zip");
        let total = snapshot.download_bytes();
        download::to_path(
            download::client()?.get(self.url),
            &path,
            download::DownloadPolicy {
                maximum_bytes: self.bytes,
                required_bytes: Some(self.bytes),
                cancellation_message: RESOURCE_PREPARATION_CANCELLED,
            },
            cancellation,
            &mut |bytes, _| {
                progress(ResourcePreparationProgress {
                    stage: ResourcePreparationStage::Downloading,
                    downloaded: bytes,
                    total: Some(total),
                    percent: Some((bytes * 100 / total).min(100) as u8),
                })
            },
        )
        .await
        .map_err(|error| error.message())?;
        let target = directory.to_owned();
        let token = cancellation.clone();
        crate::foundation::blocking::run(move || {
            self.extract(&path, &target, snapshot.models(), &token)
        })
        .await
    }

    fn extract(
        self,
        archive_path: &Path,
        directory: &Path,
        kinds: &[ModelKind],
        cancellation: &CancellationToken,
    ) -> Result<(), String> {
        if !verification::verify_cancellable(
            archive_path,
            self.sha256,
            cancellation,
            RESOURCE_PREPARATION_CANCELLED,
        )? {
            return Err("Model release archive digest differs".into());
        }
        let mut archive =
            zip::ZipArchive::new(fs::File::open(archive_path).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        for kind in kinds.iter().copied().filter(|kind| self.owns(*kind)) {
            if cancellation.is_cancelled() {
                return Err(RESOURCE_PREPARATION_CANCELLED.into());
            }
            let descriptor = catalog::descriptor(kind);
            let member_name = format!("{}/{}", self.directory, descriptor.file);
            let mut member = archive
                .by_name(&member_name)
                .map_err(|error| error.to_string())?;
            if !member.is_file() || member.is_symlink() || member.size() != descriptor.download_size
            {
                return Err("Model release member identity differs".into());
            }
            let mut file = fs::File::create(directory.join(descriptor.file))
                .map_err(|error| error.to_string())?;
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                if cancellation.is_cancelled() {
                    return Err(RESOURCE_PREPARATION_CANCELLED.into());
                }
                let count = member
                    .read(&mut buffer)
                    .map_err(|error| error.to_string())?;
                if count == 0 {
                    break;
                }
                file.write_all(&buffer[..count])
                    .map_err(|error| error.to_string())?;
            }
            file.sync_all().map_err(|error| error.to_string())?;
        }
        drop(archive);
        fs::remove_file(archive_path).map_err(|error| error.to_string())
    }
}
