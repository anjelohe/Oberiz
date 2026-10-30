use axum::{
    Json,
    body::HttpBody,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
};
use serde::Serialize;
use std::{
    path::{Path as FsPath, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::time::{Duration, sleep};

use crate::{AppState, history};

const PREFIX: &str = "oberiz-backup-";
const EXTENSION: &str = ".sqlite3";

#[derive(Serialize)]
pub struct BackupFile {
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: u64,
}
#[derive(Serialize)]
pub struct BackupList {
    pub directory: String,
    pub backups: Vec<BackupFile>,
}
#[derive(Serialize)]
pub struct BackupResult {
    pub backup: BackupFile,
    pub message: &'static str,
}
#[derive(Serialize)]
pub struct RestoreResult {
    pub message: &'static str,
}

#[derive(Serialize)]
pub struct DeleteResult {
    pub message: &'static str,
}

fn internal(error: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
fn backup_directory() -> Result<PathBuf, (StatusCode, String)> {
    let directory = std::env::var_os("OBERIZ_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
        .join("backups");
    std::fs::create_dir_all(&directory).map_err(internal)?;
    Ok(directory)
}
fn timestamp_millis() -> Result<u128, (StatusCode, String)> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .map_err(internal)
}
fn filename_is_safe(filename: &str) -> bool {
    filename.starts_with(PREFIX)
        && filename.ends_with(EXTENSION)
        && filename.len() > PREFIX.len() + EXTENSION.len()
        && filename[PREFIX.len()..filename.len() - EXTENSION.len()]
            .chars()
            .all(|character| character.is_ascii_digit())
}
fn backup_info(path: &FsPath) -> Result<BackupFile, (StatusCode, String)> {
    let metadata = std::fs::metadata(path).map_err(internal)?;
    let created_at = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis() as u64)
        .unwrap_or_default();
    Ok(BackupFile {
        filename: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_owned(),
        size_bytes: metadata.len(),
        created_at,
    })
}

pub async fn list_backups(
    State(_state): State<AppState>,
) -> Result<Json<BackupList>, (StatusCode, String)> {
    let directory = backup_directory()?;
    let mut backups = Vec::new();
    for entry in std::fs::read_dir(&directory).map_err(internal)? {
        let path = entry.map_err(internal)?.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(filename_is_safe)
        {
            backups.push(backup_info(&path)?);
        }
    }
    backups.sort_by_key(|backup| std::cmp::Reverse(backup.created_at));
    Ok(Json(BackupList {
        directory: directory.display().to_string(),
        backups,
    }))
}

async fn create_backup_file(state: &AppState) -> Result<BackupFile, (StatusCode, String)> {
    let path = backup_directory()?.join(format!("{PREFIX}{}{}", timestamp_millis()?, EXTENSION));
    // VACUUM INTO creates a self-contained, consistent snapshot, including WAL changes.
    sqlx::query("VACUUM INTO ?")
        .bind(path.to_string_lossy().to_string())
        .execute(&state.db)
        .await
        .map_err(internal)?;
    store_custom_definitions(state, &path).await?;
    backup_info(&path)
}

/// Table inside a backup with the user's own indexer definitions (custom YAML files). They live
/// in the config folder, not in the database, so a restore on a new installation lost them.
const CUSTOM_TABLE: &str = "oberiz_backup_custom_indexers";

async fn custom_folder(state: &AppState) -> Result<PathBuf, (StatusCode, String)> {
    Ok(PathBuf::from(
        crate::settings::get_value(&state.db, "paths.custom_indexers")
            .await
            .map_err(internal)?
            .unwrap_or_else(crate::settings::default_custom_indexers_path),
    ))
}

fn definition_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    (lower.ends_with(".yml") || lower.ends_with(".yaml"))
        && name.len() <= 128
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        && !name.starts_with('.')
}

async fn store_custom_definitions(state: &AppState, backup: &FsPath) -> Result<(), (StatusCode, String)> {
    use sqlx::Connection;
    let folder = custom_folder(state).await?;
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&folder) {
        for entry in entries.flatten().take(500) {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !definition_name(&name) || !entry.file_type().is_ok_and(|t| t.is_file()) { continue; }
            if let Ok(bytes) = std::fs::read(entry.path()) && bytes.len() <= 1024 * 1024 {
                files.push((name, bytes));
            }
        }
    }
    let url = format!("sqlite://{}", backup.to_string_lossy());
    let mut db = sqlx::SqliteConnection::connect(&url).await.map_err(internal)?;
    sqlx::query(&format!("CREATE TABLE {CUSTOM_TABLE}(name TEXT PRIMARY KEY, content BLOB NOT NULL)")).execute(&mut db).await.map_err(internal)?;
    for (name, bytes) in files {
        sqlx::query(&format!("INSERT INTO {CUSTOM_TABLE}(name,content) VALUES(?,?)")).bind(name).bind(bytes).execute(&mut db).await.map_err(internal)?;
    }
    db.close().await.map_err(internal)?;
    Ok(())
}

async fn apply_retention(state: &AppState) -> Result<(), (StatusCode, String)> {
    let keep = crate::settings::get_value(&state.db, "backup.retention_count")
        .await
        .map_err(internal)?
        .and_then(|item| item.parse::<usize>().ok())
        .unwrap_or(7)
        .clamp(1, 100);
    let directory = backup_directory()?;
    let mut rows = Vec::new();
    for entry in std::fs::read_dir(&directory).map_err(internal)? {
        let path = entry.map_err(internal)?.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(filename_is_safe)
        {
            rows.push((backup_info(&path)?, path));
        }
    }
    rows.sort_by_key(|(backup, _)| std::cmp::Reverse(backup.created_at));
    for (_, path) in rows.into_iter().skip(keep) {
        std::fs::remove_file(path).map_err(internal)?;
    }
    Ok(())
}

pub async fn create_backup(
    State(state): State<AppState>,
) -> Result<Json<BackupResult>, (StatusCode, String)> {
    let backup = create_backup_file(&state).await?;
    history::record(
        &state.db,
        "backup_created",
        "Database backup created",
        Some(&backup.filename),
        "info",
    )
    .await;
    Ok(Json(BackupResult {
        backup,
        message: "Backup created successfully.",
    }))
}

pub async fn download_backup(
    Path(filename): Path<String>,
) -> Result<(HeaderMap, Vec<u8>), (StatusCode, String)> {
    if !filename_is_safe(&filename) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid backup filename".to_owned(),
        ));
    }
    let path = backup_directory()?.join(&filename);
    let contents = tokio::fs::read(path).await.map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            (StatusCode::NOT_FOUND, "Backup not found".to_owned())
        } else {
            internal(error)
        }
    })?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.sqlite3"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")).map_err(internal)?,
    );
    Ok((headers, contents))
}

pub(crate) async fn receive_backup(
    request: axum::extract::Request,
    directory: PathBuf,
) -> Result<PathBuf, (StatusCode, String)> {
    use tokio::io::AsyncWriteExt;
    static UPLOAD: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);
    let _permit = UPLOAD.try_acquire().map_err(|_| {
        (
            StatusCode::TOO_MANY_REQUESTS,
            "Backup upload already running".into(),
        )
    })?;
    let limit = 512 * 1024 * 1024usize;
    if request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .is_some_and(|n| n > limit)
    {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "Backup exceeds 512 MiB".into(),
        ));
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(internal)?
        .as_nanos();
    let path = directory.join(format!("{PREFIX}{stamp}{EXTENSION}"));
    let temporary = path.with_extension("upload");
    let result = async {
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .await
            .map_err(internal)?;
        let mut body = request.into_body();
        let mut total = 0usize;
        let mut signature: Vec<u8> = Vec::new();
        while let Some(frame) =
            std::future::poll_fn(|cx| std::pin::Pin::new(&mut body).poll_frame(cx)).await
        {
            let frame =
                frame.map_err(|_| (StatusCode::BAD_REQUEST, "Invalid upload stream".into()))?;
            if let Ok(bytes) = frame.into_data() {
                total = total
                    .checked_add(bytes.len())
                    .ok_or((StatusCode::PAYLOAD_TOO_LARGE, "Backup too large".into()))?;
                if total > limit {
                    return Err((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "Backup exceeds 512 MiB".into(),
                    ));
                }
                signature.extend(bytes.iter().take(16 - signature.len()));
                if signature.len() == 16 && signature != b"SQLite format 3\0" {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Upload a valid SQLite database backup.".into(),
                    ));
                }
                file.write_all(&bytes).await.map_err(internal)?;
            }
        }
        if signature.len() != 16 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Upload a valid SQLite database backup.".into(),
            ));
        }
        file.flush().await.map_err(internal)?;
        drop(file);
        tokio::fs::rename(&temporary, &path)
            .await
            .map_err(internal)?;
        Ok::<(), (StatusCode, String)>(())
    };
    let result = tokio::time::timeout(Duration::from_secs(300), result)
        .await
        .unwrap_or_else(|_| {
            Err((
                StatusCode::REQUEST_TIMEOUT,
                "Backup upload timed out".into(),
            ))
        });
    if result.is_err() {
        let _ = tokio::fs::remove_file(&temporary).await;
    }
    result?;
    Ok(path)
}

pub async fn upload_backup(
    State(state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<BackupResult>, (StatusCode, String)> {
    let path = receive_backup(request, backup_directory()?).await?;
    let backup = backup_info(&path)?;
    history::record(
        &state.db,
        "backup_uploaded",
        "Database backup uploaded",
        Some(&backup.filename),
        "info",
    )
    .await;
    Ok(Json(BackupResult {
        backup,
        message: "Backup uploaded successfully.",
    }))
}

pub async fn delete_backup(
    Path(filename): Path<String>,
) -> Result<Json<DeleteResult>, (StatusCode, String)> {
    if !filename_is_safe(&filename) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid backup filename".to_owned(),
        ));
    }
    let path = backup_directory()?.join(&filename);
    std::fs::remove_file(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            (StatusCode::NOT_FOUND, "Backup not found".to_owned())
        } else {
            internal(error)
        }
    })?;
    Ok(Json(DeleteResult {
        message: "Backup deleted.",
    }))
}

pub fn spawn_scheduler(state: AppState) {
    tokio::spawn(async move {
        loop {
            let hours = crate::settings::get_value(&state.db, "backup.interval_hours")
                .await
                .ok()
                .flatten()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(24)
                .clamp(1, 720);
            sleep(Duration::from_secs(hours * 3600)).await;
            let enabled = crate::settings::get_value(&state.db, "backup.enabled")
                .await
                .ok()
                .flatten()
                .as_deref()
                == Some("true");
            if enabled && let Ok(backup) = create_backup_file(&state).await {
                let _ = apply_retention(&state).await;
                history::record(
                    &state.db,
                    "backup_scheduled",
                    "Scheduled database backup created",
                    Some(&backup.filename),
                    "info",
                )
                .await;
            }
        }
    });
}

async fn table_names(
    connection: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
    schema: &str,
) -> Result<Vec<String>, (StatusCode, String)> {
    let query = match schema {
        "main" => {
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name <> 'oberiz_backup_custom_indexers' ORDER BY name"
        }
        "restore" => {
            "SELECT name FROM restore.sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name <> 'oberiz_backup_custom_indexers' ORDER BY name"
        }
        _ => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unsupported database schema".to_owned(),
            ));
        }
    };
    sqlx::query_scalar(query)
        .fetch_all(&mut **connection)
        .await
        .map_err(internal)
}
fn quoted(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

const SECURITY_FILTER: &str = "key LIKE 'auth.%' OR key LIKE 'api.%' OR key LIKE 'qbittorrent.%' OR key LIKE 'cinetta.%' OR key LIKE 'tmdb.%' OR key LIKE 'tvdb.%' OR key='overseerr.compat_enabled'";

pub(crate) async fn security_snapshot(
    connection: &mut sqlx::SqliteConnection,
) -> Result<Vec<(String, String)>, (StatusCode, String)> {
    sqlx::query_as(&format!(
        "SELECT key,value FROM settings WHERE {SECURITY_FILTER}"
    ))
    .fetch_all(connection)
    .await
    .map_err(internal)
}
pub(crate) async fn preserve_security(
    connection: &mut sqlx::SqliteConnection,
    values: Vec<(String, String)>,
) -> Result<(), (StatusCode, String)> {
    sqlx::query(&format!("DELETE FROM settings WHERE {SECURITY_FILTER}"))
        .execute(&mut *connection)
        .await
        .map_err(internal)?;
    for (key, value) in values {
        sqlx::query("INSERT INTO settings(key,value) VALUES(?,?)")
            .bind(key)
            .bind(value)
            .execute(&mut *connection)
            .await
            .map_err(internal)?;
    }
    Ok(())
}

pub async fn restore_backup(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<RestoreResult>, (StatusCode, String)> {
    if !filename_is_safe(&filename) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid backup filename".to_owned(),
        ));
    }
    let path = backup_directory()?.join(&filename);
    if !path.is_file() {
        return Err((StatusCode::NOT_FOUND, "Backup not found".to_owned()));
    }
    // A safety net for the classic self-hosted mistake of restoring the wrong
    // file: always leave a way back, even if the chosen backup turns out to
    // be the wrong one. Abort rather than restore if it can't be created.
    let safety_backup = create_backup_file(&state).await?;
    history::record(
        &state.db,
        "backup_created",
        "Pre-restore safety backup created",
        Some(&safety_backup.filename),
        "info",
    )
    .await;
    // Read before holding a connection: the pool may have a single one.
    let custom_dir = custom_folder(&state).await?;
    let mut connection = state.db.acquire().await.map_err(internal)?;
    let result = async {
        sqlx::query("PRAGMA foreign_keys = OFF").execute(&mut *connection).await.map_err(internal)?;
        sqlx::query("ATTACH DATABASE ? AS restore").bind(path.to_string_lossy().to_string()).execute(&mut *connection).await.map_err(internal)?;
        let current_tables = table_names(&mut connection, "main").await?;
        if current_tables != table_names(&mut connection, "restore").await? { return Err((StatusCode::CONFLICT, "This backup belongs to a different Oberiz database schema. Update Oberiz or choose a compatible backup.".to_owned())); }
        sqlx::query("BEGIN IMMEDIATE").execute(&mut *connection).await.map_err(internal)?;
        let security_settings = security_snapshot(&mut connection).await?;
        // Tracker credentials are security state: keep the current ones so an
        // old backup never revives keys that were rotated or removed. A fresh
        // installation has none to protect, so then the backup's are restored.
        let current_indexers: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM main.indexer_configs")
            .fetch_one(&mut *connection).await.map_err(internal)?;
        let restore_indexers = current_indexers == 0;
        for table in &current_tables {
            if table == "indexer_configs" && !restore_indexers { continue; }
            sqlx::query(&format!("DELETE FROM {}", quoted(table))).execute(&mut *connection).await.map_err(internal)?;
        }
        for table in &current_tables {
            if table == "auth_sessions" || (table == "indexer_configs" && !restore_indexers) {
                // Session state is security state, not restorable data: a
                // backup can hold a session that was later revoked by logout
                // or a password change, and bringing it back would silently
                // undo that revocation, handing out admin access again
                // through a token the user believed was dead. The DELETE
                // above already cleared every current session (including
                // the one making this request), so skipping the restore
                // here just means everyone — this admin included — logs in
                // fresh afterward instead of a stale token staying valid.
                continue;
            }
            sqlx::query(&format!("INSERT INTO {} SELECT * FROM restore.{}", quoted(table), quoted(table))).execute(&mut *connection).await.map_err(internal)?;
        }
        // Backups restore application data, never silently roll security back.
        // Keep entire connection settings together to avoid sending a current
        // password to an old destination from the backup.
        preserve_security(&mut connection, security_settings).await?;
        let violation: Option<String> = sqlx::query_scalar("PRAGMA foreign_key_check").fetch_optional(&mut *connection).await.map_err(internal)?;
        if violation.is_some() { return Err((StatusCode::CONFLICT, "Backup failed the database integrity check; nothing was restored.".to_owned())); }
        sqlx::query("COMMIT").execute(&mut *connection).await.map_err(internal)?;
        Ok::<(), (StatusCode, String)>(())
    }.await;
    if result.is_err() {
        let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
    }
    let mut recovered: Vec<(String, Vec<u8>)> = Vec::new();
    if result.is_ok() {
        let present: Option<String> = sqlx::query_scalar("SELECT name FROM restore.sqlite_master WHERE type='table' AND name='oberiz_backup_custom_indexers'")
            .fetch_optional(&mut *connection).await.unwrap_or(None);
        if present.is_some() {
            recovered = sqlx::query_as("SELECT name, content FROM restore.oberiz_backup_custom_indexers LIMIT 500")
                .fetch_all(&mut *connection).await.unwrap_or_default();
        }
    }
    let _ = sqlx::query("DETACH DATABASE restore")
        .execute(&mut *connection)
        .await;
    let _ = sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *connection)
        .await;
    result?;
    if !recovered.is_empty() {
        let folder = custom_dir;
        let _ = std::fs::create_dir_all(&folder);
        for (name, bytes) in recovered {
            let target = folder.join(&name);
            if definition_name(&name) && bytes.len() <= 1024 * 1024 && !target.exists() {
                let _ = std::fs::write(&target, bytes);
            }
        }
    }
    history::record(
        &state.db,
        "backup_restored",
        "Database backup restored",
        Some(&filename),
        "warning",
    )
    .await;
    Ok(Json(RestoreResult {
        message: "Backup restored successfully.",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn indexers(state: &AppState) -> Vec<String> {
        sqlx::query_scalar("SELECT indexer_id FROM indexer_configs ORDER BY indexer_id")
            .fetch_all(&state.db)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn restore_brings_back_indexers_only_when_none_are_configured() {
        let root = std::env::temp_dir().join(format!("oberiz-restore-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        // Only this test reads OBERIZ_DATA_DIR (backup_directory).
        unsafe { std::env::set_var("OBERIZ_DATA_DIR", &root) };
        let url = format!("sqlite://{}?mode=rwc", root.join("oberiz.db").display());
        let db = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();
        let state = AppState {
            db,
            http: reqwest::Client::new(),
            automation_runtime: Default::default(),
            download_client: Arc::new(crate::download_client::QBittorrentClient),
            torrent_list_cache: Arc::new(Mutex::new(None)),
            grab_lock: Arc::new(Mutex::new(())),
        };
        sqlx::query("INSERT INTO indexer_configs(indexer_id,enabled,config_json) VALUES('from-backup',1,'{}')")
            .execute(&state.db).await.unwrap();
        // The user's own definitions (config folder, not database) travel inside the copy.
        let custom = root.join("custom");
        std::fs::create_dir_all(&custom).unwrap();
        std::fs::write(custom.join("mine-custom.yml"), b"id: mine-custom\n").unwrap();
        crate::settings::set_value(&state.db, "paths.custom_indexers", &custom.to_string_lossy()).await.unwrap();
        let backup = create_backup_file(&state).await.unwrap();
        std::fs::remove_file(custom.join("mine-custom.yml")).unwrap();

        // Fresh installation: nothing to protect, so the backup's indexers return.
        sqlx::query("DELETE FROM indexer_configs").execute(&state.db).await.unwrap();
        restore_backup(State(state.clone()), Path(backup.filename.clone())).await.unwrap();
        assert_eq!(indexers(&state).await, ["from-backup"]);
        assert_eq!(std::fs::read(custom.join("mine-custom.yml")).unwrap(), b"id: mine-custom\n", "custom definition recovered");

        // Indexers already configured: the current ones are kept.
        sqlx::query("UPDATE indexer_configs SET indexer_id='current'").execute(&state.db).await.unwrap();
        restore_backup(State(state.clone()), Path(backup.filename)).await.unwrap();
        assert_eq!(indexers(&state).await, ["current"]);

        state.db.close().await;
        let _ = std::fs::remove_dir_all(&root);
    }
}
