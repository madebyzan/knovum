use super::super::super::content::{self, PackageFiles};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio_util::sync::CancellationToken;

pub(super) fn read(
    url: &str,
    reference: &str,
    subdirectory: &str,
    addresses: Vec<std::net::SocketAddr>,
    cancellation: CancellationToken,
) -> Result<(PackageFiles, String), String> {
    if !subdirectory.is_empty() {
        content::validate_path(subdirectory)?;
    }
    let directory =
        std::env::temp_dir().join(format!("knovum-plugin-git-{}", uuid::Uuid::new_v4()));
    let interrupt = Arc::new(AtomicBool::new(false));
    let (done, finished) = std::sync::mpsc::channel();
    let watchdog_path = directory.clone();
    let watchdog_interrupt = interrupt.clone();
    let watchdog = std::thread::spawn(move || {
        let started = Instant::now();
        while finished.recv_timeout(Duration::from_millis(200)).is_err() {
            if cancellation.is_cancelled()
                || started.elapsed() > Duration::from_secs(120)
                || disk_bytes(&watchdog_path) > 256 * 1024 * 1024
            {
                watchdog_interrupt.store(true, Ordering::Relaxed);
                break;
            }
        }
    });
    let result = read_objects(
        url,
        reference,
        subdirectory,
        &directory,
        &interrupt,
        addresses,
    );
    let _ = done.send(());
    let _ = watchdog.join();
    let _ = std::fs::remove_dir_all(directory);
    result
}

fn disk_bytes(directory: &Path) -> u64 {
    walkdir::WalkDir::new(directory)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok())
        .map(|metadata| metadata.len())
        .sum()
}

fn read_objects(
    url: &str,
    reference: &str,
    subdirectory: &str,
    directory: &Path,
    interrupt: &AtomicBool,
    addresses: Vec<std::net::SocketAddr>,
) -> Result<(PackageFiles, String), String> {
    let origin = url::Url::parse(url).map_err(|error| error.to_string())?;
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .resolve_to_addrs(
            origin.host_str().ok_or("git-source-host-required")?,
            &addresses,
        )
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|error| error.to_string())?;
    let mut fetch = gix::clone::PrepareFetch::new(
        url,
        directory,
        gix::create::Kind::Bare,
        Default::default(),
        gix::open::Options::isolated(),
    )
    .map_err(|error| error.to_string())?
    .configure_connection(move |connection| {
        let transport = gix::protocol::transport::client::blocking_io::http::connect_http(
            super::git_http::GitHttp {
                client: client.clone(),
                origin: origin.clone(),
            },
            gix::url::parse(origin.as_str()).map_err(|error| error.erased())?,
            gix::protocol::transport::Protocol::V2,
            false,
        );
        *connection.transport_mut() = Box::new(transport);
        Ok(())
    })
    .with_in_memory_config_overrides([
        "credential.helper=",
        "credential.interactive=false",
        "http.followRedirects=false",
        "http.lowSpeedLimit=1024",
        "http.lowSpeedTime=30",
    ])
    .with_revision(Some(reference))
    .map_err(|error| error.to_string())?
    .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(
        std::num::NonZeroU32::new(1).expect("depth"),
    ));
    let (repository, _) = fetch
        .fetch_only(gix::progress::Discard, interrupt)
        .map_err(|error| error.to_string())?;
    let commit = repository
        .head_commit()
        .map_err(|error| error.to_string())?;
    let revision = commit.id.to_string();
    let mut tree = commit.tree().map_err(|error| error.to_string())?;
    if !subdirectory.is_empty() {
        let entry = tree
            .lookup_entry_by_path(subdirectory)
            .map_err(|error| error.to_string())?
            .ok_or("git-plugin-subdirectory-missing")?;
        tree = entry
            .object()
            .map_err(|error| error.to_string())?
            .try_into_tree()
            .map_err(|error| error.to_string())?;
    }
    let mut files = PackageFiles::new();
    collect(&tree, "", &mut files, &mut 0usize, interrupt)?;
    content::validate_files(&files)?;
    Ok((files, revision))
}

fn collect(
    tree: &gix::Tree<'_>,
    prefix: &str,
    files: &mut PackageFiles,
    total: &mut usize,
    interrupt: &AtomicBool,
) -> Result<(), String> {
    for entry in tree.iter() {
        if interrupt.load(Ordering::Relaxed) {
            return Err(super::super::operations::CANCELLED.into());
        }
        let entry = entry.map_err(|error| error.to_string())?;
        let name = std::str::from_utf8(entry.filename()).map_err(|_| "git-path-must-be-utf8")?;
        let path = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        content::validate_path(&path)?;
        let object = entry.object().map_err(|error| error.to_string())?;
        if entry.mode().is_tree() {
            collect(
                &object.try_into_tree().map_err(|error| error.to_string())?,
                &path,
                files,
                total,
                interrupt,
            )?;
        } else if entry.mode().is_blob() {
            if object.data.len() > content::MAX_FILE_BYTES {
                return Err("package-file-size-limit".into());
            }
            *total += object.data.len();
            if *total > content::MAX_PACKAGE_BYTES || files.len() >= content::MAX_FILES {
                return Err("package-size-limit-exceeded".into());
            }
            files.insert(path, object.data.to_vec());
        } else {
            return Err("git-symlinks-and-submodules-not-supported".into());
        }
    }
    Ok(())
}
