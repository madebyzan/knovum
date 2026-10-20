use sha2::{Digest, Sha256};
use sqlx::{migrate::Migrator, sqlite::SqliteConnectOptions, Connection, Row, SqliteConnection};
use std::{collections::BTreeMap, path::Path};

static SPACES: Migrator = sqlx::migrate!("src/spaces/db/migrations");
static EXTENSIONS: Migrator = sqlx::migrate!("src/extensions/migrations");
static AI_USAGE: Migrator = sqlx::migrate!("src/ai/usage/migrations");

pub const DATABASES: &[(&str, &str)] = &[
    ("spaces.db", "data/config/spaces.db"),
    ("extensions.db", "data/config/extensions.db"),
    ("ai-usage.db", "data/config/ai-usage.db"),
];

pub(super) fn migrator(name: &str) -> &'static Migrator {
    match name {
        "spaces.db" => &SPACES,
        "extensions.db" => &EXTENSIONS,
        "ai-usage.db" => &AI_USAGE,
        _ => unreachable!("database names are static"),
    }
}

pub fn baselines() -> BTreeMap<String, String> {
    DATABASES
        .iter()
        .map(|(name, _)| {
            let mut digest = Sha256::new();
            for migration in migrator(name).migrations.iter() {
                digest.update(migration.version.to_le_bytes());
                digest.update(migration.checksum.as_ref());
            }
            ((*name).into(), hex::encode(digest.finalize()))
        })
        .collect()
}

pub async fn snapshot(name: &str, source: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        std::fs::remove_file(destination).map_err(|error| error.to_string())?;
    }
    let mut connection = connect(source).await?;
    validate_connection(name, &mut connection).await?;
    if name == "spaces.db" {
        crate::spaces::backup::assert_quiescent(&mut connection).await?;
    }
    sqlx::query("VACUUM INTO ?")
        .bind(destination.to_string_lossy().as_ref())
        .execute(&mut connection)
        .await
        .map_err(|error| format!("{name} snapshot failed: {error}"))?;
    drop(connection);
    if name == "spaces.db" {
        let mut copy = connect(destination).await?;
        crate::spaces::backup::reindex_search(&mut copy).await?;
        drop(copy);
    }
    validate(name, destination).await
}

pub async fn validate(name: &str, path: &Path) -> Result<(), String> {
    let mut connection = connect(path).await?;
    validate_connection(name, &mut connection).await?;
    if name == "spaces.db" {
        crate::spaces::backup::assert_quiescent(&mut connection).await?;
        crate::spaces::backup::assert_search_aligned(&mut connection).await?;
    }
    Ok(())
}

async fn connect(path: &Path) -> Result<SqliteConnection, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("Database could not be inspected: {error}"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "Database is not a regular file: {}",
            path.display()
        ));
    }
    SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(path))
        .await
        .map_err(|error| error.to_string())
}

async fn validate_connection(name: &str, connection: &mut SqliteConnection) -> Result<(), String> {
    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&mut *connection)
        .await
        .map_err(|error| error.to_string())?;
    if integrity != "ok" {
        return Err(format!("{name} failed integrity validation: {integrity}"));
    }
    let rows =
        sqlx::query("SELECT version, success, checksum FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&mut *connection)
            .await
            .map_err(|error| format!("{name} has no valid baseline: {error}"))?;
    let expected = &migrator(name).migrations;
    if rows.len() != expected.len() {
        return Err(format!(
            "{name} schema baseline does not match this release"
        ));
    }
    for (row, migration) in rows.iter().zip(expected.iter()) {
        let version: i64 = row.try_get("version").map_err(|error| error.to_string())?;
        let success: bool = row.try_get("success").map_err(|error| error.to_string())?;
        let checksum: Vec<u8> = row.try_get("checksum").map_err(|error| error.to_string())?;
        if version != migration.version || !success || checksum != migration.checksum.as_ref() {
            return Err(format!(
                "{name} schema baseline does not match this release"
            ));
        }
    }
    Ok(())
}
