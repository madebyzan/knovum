use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use super::super::{send_progress, validation, SpacePackageProgress};
use super::planning::AssetPlan;

pub(super) fn write(
    file: fs::File,
    manifest: &str,
    index: &[u8],
    plans: &[AssetPlan],
    progress: &Channel<SpacePackageProgress>,
    total: u64,
    cancellation: &CancellationToken,
) -> Result<(), crate::app_error::AppError> {
    let mut archive = ArchiveWriter::new(file, progress, total, cancellation);
    archive.write_bytes("manifest.json", manifest.as_bytes())?;
    for plan in plans {
        archive.check_cancelled()?;
        archive.write_asset(plan)?;
    }
    archive.write_bytes("assets-index.json", index)?;
    archive.finish()
}

struct ArchiveWriter<'a> {
    zip: ZipWriter<fs::File>,
    options: SimpleFileOptions,
    processed: u64,
    total: u64,
    progress: &'a Channel<SpacePackageProgress>,
    cancellation: &'a CancellationToken,
}

impl<'a> ArchiveWriter<'a> {
    fn new(
        file: fs::File,
        progress: &'a Channel<SpacePackageProgress>,
        total: u64,
        cancellation: &'a CancellationToken,
    ) -> Self {
        Self {
            zip: ZipWriter::new(file),
            options: SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
            processed: 0,
            total,
            progress,
            cancellation,
        }
    }

    fn check_cancelled(&self) -> Result<(), crate::app_error::AppError> {
        super::super::control::check(self.cancellation)
    }

    fn write_asset(&mut self, plan: &AssetPlan) -> Result<(), crate::app_error::AppError> {
        if plan.source.is_file() {
            return self.write_file(&plan.source, &format!("assets/{}/content", plan.token));
        }
        let mut stack = vec![(plan.source.clone(), PathBuf::new())];
        while let Some((directory, relative)) = stack.pop() {
            self.check_cancelled()?;
            let mut empty = true;
            for entry in fs::read_dir(&directory).map_err(|error| error.to_string())? {
                self.check_cancelled()?;
                empty = false;
                let entry = entry.map_err(|error| error.to_string())?;
                let meta = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
                let next = relative.join(entry.file_name());
                if meta.is_dir() {
                    stack.push((entry.path(), next));
                } else if meta.is_file() {
                    self.write_file(&entry.path(), &archive_name(&plan.token, &next)?)?;
                }
            }
            if empty && !relative.as_os_str().is_empty() {
                self.zip
                    .add_directory(
                        format!("{}/", archive_name(&plan.token, &relative)?),
                        self.options,
                    )
                    .map_err(|error| error.to_string())?;
            }
        }
        Ok(())
    }

    fn write_file(&mut self, source: &Path, name: &str) -> Result<(), crate::app_error::AppError> {
        self.zip
            .start_file(name, self.options)
            .map_err(|error| error.to_string())?;
        let mut file = fs::File::open(source).map_err(|error| error.to_string())?;
        let mut buffer = [0_u8; 1024 * 1024];
        loop {
            self.check_cancelled()?;
            let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            self.write_chunk(
                &buffer[..count],
                "A Space asset changed size while the Space package was being written",
            )?;
        }
        Ok(())
    }

    fn write_bytes(&mut self, name: &str, bytes: &[u8]) -> Result<(), crate::app_error::AppError> {
        self.check_cancelled()?;
        self.zip
            .start_file(name, self.options)
            .map_err(|error| error.to_string())?;
        for chunk in bytes.chunks(1024 * 1024) {
            self.check_cancelled()?;
            self.write_chunk(chunk, "Space package output exceeded its validated size")?;
        }
        Ok(())
    }

    fn write_chunk(
        &mut self,
        chunk: &[u8],
        size_error: &str,
    ) -> Result<(), crate::app_error::AppError> {
        let next = self
            .processed
            .checked_add(chunk.len() as u64)
            .ok_or("Space package size overflow")?;
        if next > self.total {
            return Err(size_error.into());
        }
        self.zip
            .write_all(chunk)
            .map_err(|error| error.to_string())?;
        self.processed = next;
        Ok(send_progress(
            self.progress,
            "writing",
            self.processed,
            self.total,
        )?)
    }

    fn finish(self) -> Result<(), crate::app_error::AppError> {
        self.check_cancelled()?;
        let file = self.zip.finish().map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        Ok(send_progress(
            self.progress,
            "writing",
            self.total,
            self.total,
        )?)
    }
}

fn archive_name(token: &str, relative: &Path) -> Result<String, String> {
    let value = relative
        .to_str()
        .ok_or("Package asset path is not UTF-8")?
        .replace('\\', "/");
    let name = format!("assets/{token}/{value}");
    validation::validate_entry_name(&name)?;
    Ok(name)
}
