use axum::{
    Json,
    body::Bytes,
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
    backup_info(&path)
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

pub async fn upload_backup(
    State(state): State<AppState>,
    body: Bytes,
) -> Result<Json<BackupResult>, (StatusCode, String)> {
    if body.len() < 16 || &body[..16] != b"SQLite format 3\0" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Upload a valid SQLite database backup.".to_owned(),
        ));
    }
    let path = backup_directory()?.join(format!("{PREFIX}{}{}", timestamp_millis()?, EXTENSION));
    tokio::fs::write(&path, &body).await.map_err(internal)?;
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
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        }
        "restore" => {
            "SELECT name FROM restore.sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
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
    let mut connection = state.db.acquire().await.map_err(internal)?;
    let result = async {
        sqlx::query("PRAGMA foreign_keys = OFF").execute(&mut *connection).await.map_err(internal)?;
        sqlx::query("ATTACH DATABASE ? AS restore").bind(path.to_string_lossy().to_string()).execute(&mut *connection).await.map_err(internal)?;
        let current_tables = table_names(&mut connection, "main").await?;
        if current_tables != table_names(&mut connection, "restore").await? { return Err((StatusCode::CONFLICT, "This backup belongs to a different Oberiz database schema. Update Oberiz or choose a compatible backup.".to_owned())); }
        sqlx::query("BEGIN IMMEDIATE").execute(&mut *connection).await.map_err(internal)?;
        for table in &current_tables { sqlx::query(&format!("DELETE FROM {}", quoted(table))).execute(&mut *connection).await.map_err(internal)?; }
        for table in &current_tables {
            if table == "auth_sessions" {
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
        let violation: Option<String> = sqlx::query_scalar("PRAGMA foreign_key_check").fetch_optional(&mut *connection).await.map_err(internal)?;
        if violation.is_some() { return Err((StatusCode::CONFLICT, "Backup failed the database integrity check; nothing was restored.".to_owned())); }
        sqlx::query("COMMIT").execute(&mut *connection).await.map_err(internal)?;
        Ok::<(), (StatusCode, String)>(())
    }.await;
    if result.is_err() {
        let _ = sqlx::query("ROLLBACK").execute(&mut *connection).await;
    }
    let _ = sqlx::query("DETACH DATABASE restore")
        .execute(&mut *connection)
        .await;
    let _ = sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *connection)
        .await;
    result?;
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
