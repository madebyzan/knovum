use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use tauri::ipc::Channel;
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
) -> Result<(), String> {
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let mut processed = 0_u64;
    write_bytes(
        &mut zip,
        "manifest.json",
        manifest.as_bytes(),
        options,
        &mut processed,
        total,
        progress,
    )?;
    for plan in plans {
        write_asset(&mut zip, plan, options, &mut processed, total, progress)?;
    }
    write_bytes(
        &mut zip,
        "assets-index.json",
        index,
        options,
        &mut processed,
        total,
        progress,
    )?;
    let file = zip.finish().map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    send_progress(progress, "writing", total, total)
}

fn write_asset(
    zip: &mut ZipWriter<fs::File>,
    plan: &AssetPlan,
    options: SimpleFileOptions,
    processed: &mut u64,
    total: u64,
    progress: &Channel<SpacePackageProgress>,
) -> Result<(), String> {
    if plan.source.is_file() {
        return write_file(
            zip,
            &plan.source,
            &format!("assets/{}/content", plan.token),
            options,
            processed,
            total,
            progress,
        );
    }
    let mut stack = vec![(plan.source.clone(), PathBuf::new())];
    while let Some((directory, relative)) = stack.pop() {
        let mut empty = true;
        for entry in fs::read_dir(&directory).map_err(|error| error.to_string())? {
            empty = false;
            let entry = entry.map_err(|error| error.to_string())?;
            let meta = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
            let next = relative.join(entry.file_name());
            if meta.is_dir() {
                stack.push((entry.path(), next));
            } else if meta.is_file() {
                write_file(
                    zip,
                    &entry.path(),
                    &archive_name(&plan.token, &next)?,
                    options,
                    processed,
                    total,
                    progress,
                )?;
            }
        }
        if empty && !relative.as_os_str().is_empty() {
            zip.add_directory(
                format!("{}/", archive_name(&plan.token, &relative)?),
                options,
            )
            .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn write_file(
    zip: &mut ZipWriter<fs::File>,
    source: &Path,
    name: &str,
    options: SimpleFileOptions,
    processed: &mut u64,
    total: u64,
    progress: &Channel<SpacePackageProgress>,
) -> Result<(), String> {
    zip.start_file(name, options)
        .map_err(|error| error.to_string())?;
    let mut file = fs::File::open(source).map_err(|error| error.to_string())?;
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        let next = (*processed)
            .checked_add(count as u64)
            .ok_or("Space package size overflow")?;
        if next > total {
            return Err(
                "A Space asset changed size while the Space package was being written".into(),
            );
        }
        zip.write_all(&buffer[..count])
            .map_err(|error| error.to_string())?;
        *processed = next;
        send_progress(progress, "writing", *processed, total)?;
    }
    Ok(())
}

fn write_bytes(
    zip: &mut ZipWriter<fs::File>,
    name: &str,
    bytes: &[u8],
    options: SimpleFileOptions,
    processed: &mut u64,
    total: u64,
    progress: &Channel<SpacePackageProgress>,
) -> Result<(), String> {
    zip.start_file(name, options)
        .map_err(|error| error.to_string())?;
    let next = (*processed)
        .checked_add(bytes.len() as u64)
        .ok_or("Space package size overflow")?;
    if next > total {
        return Err("Space package output exceeded its validated size".into());
    }
    zip.write_all(bytes).map_err(|error| error.to_string())?;
    *processed = next;
    send_progress(progress, "writing", *processed, total)
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
