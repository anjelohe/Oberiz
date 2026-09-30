use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{
    fs::File,
    io::{Cursor, Read, Write},
    path::{Path as FsPath, PathBuf},
};
use tokio::time::{Duration, sleep};
use zip::ZipArchive;

use crate::{AppState, cardigann, history, settings};

const UPSTREAM_ARCHIVE: &str =
    "https://codeload.github.com/Prowlarr/Indexers/zip/refs/heads/master";

#[derive(Debug, Serialize)]
pub struct IndexerSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub indexer_type: Option<String>,
    pub links: Vec<String>,
    pub source: String,
    pub file_name: String,
    pub valid: bool,
    pub configured: bool,
    pub enabled: bool,
    pub priority: i64,
    pub settings_count: usize,
    pub last_status: String,
    pub last_message: Option<String>,
    pub last_latency_ms: Option<i64>,
    pub last_checked_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IndexerListResponse {
    pub status: &'static str,
    pub total: usize,
    pub upstream: usize,
    pub custom: usize,
    pub enabled: usize,
    pub invalid: usize,
    pub indexers: Vec<IndexerSummary>,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub status: &'static str,
    pub installed: usize,
    pub folder: String,
}

#[derive(Debug, Serialize)]
pub struct IndexerSetting {
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub secret: bool,
    pub configured: bool,
    pub value: Option<serde_json::Value>,
    pub default: Option<serde_json::Value>,
    pub options: Vec<(String, String)>,
}

#[derive(Debug, Serialize)]
pub struct IndexerDetail {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub indexer_type: Option<String>,
    pub language: Option<String>,
    pub links: Vec<String>,
    pub enabled: bool,
    pub settings: Vec<IndexerSetting>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigRequest {
    pub values: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct TestResponse {
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct EnableRequest {
    pub enabled: bool,
}

async fn persist_runtime(state: &AppState, id: &str, status: &str, message: &str, latency: i64) {
    let _=sqlx::query(r#"
        INSERT INTO indexer_runtime(indexer_id,last_status,last_message,last_latency_ms,last_checked_at,updated_at)
        VALUES (?,?,?,?,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)
        ON CONFLICT(indexer_id) DO UPDATE SET
          last_status=excluded.last_status,
          last_message=excluded.last_message,
          last_latency_ms=excluded.last_latency_ms,
          last_checked_at=CURRENT_TIMESTAMP,
          updated_at=CURRENT_TIMESTAMP
    "#)
        .bind(id).bind(status).bind(message).bind(latency)
        .execute(&state.db).await;
}

async fn run_health_check(state: &AppState) {
    let ids = sqlx::query_scalar::<_, String>(
        "SELECT indexer_id FROM indexer_configs WHERE enabled=1 ORDER BY indexer_id",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for id in ids {
        let started = std::time::Instant::now();
        match cardigann::test_indexer(state, &id).await {
            Ok(message) => {
                persist_runtime(
                    state,
                    &id,
                    "online",
                    &message,
                    started.elapsed().as_millis() as i64,
                )
                .await;
            }
            Err(error) => {
                persist_runtime(
                    state,
                    &id,
                    "offline",
                    &error,
                    started.elapsed().as_millis() as i64,
                )
                .await;
            }
        }
    }
}

pub fn spawn_health_scheduler(state: AppState) {
    tokio::spawn(async move {
        sleep(Duration::from_secs(5)).await;
        loop {
            run_health_check(&state).await;
            sleep(Duration::from_secs(15 * 60)).await;
        }
    });
}

pub async fn remove_indexer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("DELETE FROM indexer_configs WHERE indexer_id=?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("DELETE FROM indexer_runtime WHERE indexer_id=?")
        .bind(&id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    history::record(
        &state.db,
        "indexer.removed",
        &id,
        Some("Removed from configured indexers. Definition and seed policy preserved."),
        "info",
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_indexers(
    State(state): State<AppState>,
) -> Result<Json<IndexerListResponse>, (StatusCode, String)> {
    Ok(Json(scan_all(&state).await?))
}

pub async fn sync_upstream(
    State(state): State<AppState>,
) -> Result<Json<SyncResponse>, (StatusCode, String)> {
    let target = upstream_folder(&state).await?;
    tokio::fs::create_dir_all(&target).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("No se pudo crear {}: {e}", target.display()),
        )
    })?;

    let response = state.http.get(UPSTREAM_ARCHIVE).send().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("No se pudo descargar Prowlarr/Indexers: {e}"),
        )
    })?;

    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("GitHub devolvió HTTP {}", response.status()),
        ));
    }

    let bytes = response.bytes().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("No se pudo leer el ZIP: {e}"),
        )
    })?;

    let target_clone = target.clone();
    let installed = tokio::task::spawn_blocking(move || extract_v11(bytes.to_vec(), &target_clone))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error en extracción: {e}"),
            )
        })?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    history::record(
        &state.db,
        "indexers.synced",
        "Prowlarr/Indexers sincronizado",
        Some(&format!("{installed} definiciones v11 instaladas")),
        "info",
    )
    .await;

    Ok(Json(SyncResponse {
        status: "ok",
        installed,
        folder: target.display().to_string(),
    }))
}

pub async fn set_enabled(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<EnableRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query(
        r#"
        INSERT INTO indexer_configs(indexer_id, enabled, config_json, updated_at)
        VALUES (?, ?, '{}', CURRENT_TIMESTAMP)
        ON CONFLICT(indexer_id) DO UPDATE SET
            enabled=excluded.enabled,
            updated_at=CURRENT_TIMESTAMP
        "#,
    )
    .bind(&id)
    .bind(payload.enabled)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_indexer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<IndexerDetail>, (StatusCode, String)> {
    let (yaml, _) = load_yaml_by_id(&state, &id).await?;
    let name = yaml_scalar(mapping_value(&yaml, "name")).unwrap_or_else(|| id.clone());
    let description = yaml_scalar(mapping_value(&yaml, "description"));
    let indexer_type = yaml_scalar(mapping_value(&yaml, "type"));
    let language = yaml_scalar(mapping_value(&yaml, "language"));
    let links = yaml_list_value(mapping_value(&yaml, "links"));

    let row: Option<(bool, String)> =
        sqlx::query_as("SELECT enabled, config_json FROM indexer_configs WHERE indexer_id=?")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let enabled = row.as_ref().map(|r| r.0).unwrap_or(false);
    let current: serde_json::Map<String, serde_json::Value> = row
        .as_ref()
        .and_then(|r| serde_json::from_str::<serde_json::Value>(&r.1).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    let mut settings_out = Vec::new();
    if let Some(seq) = mapping_value(&yaml, "settings").and_then(Value::as_sequence) {
        for item in seq {
            let Some(name) = yaml_scalar(mapping_value(item, "name")) else {
                continue;
            };
            let field_type =
                yaml_scalar(mapping_value(item, "type")).unwrap_or_else(|| "text".into());
            if field_type.starts_with("info") {
                continue;
            }
            let label = yaml_scalar(mapping_value(item, "label")).unwrap_or_else(|| name.clone());
            let secret = is_secret(&name, &field_type);
            let configured = current
                .get(&name)
                .map(|v| !v.is_null() && v.as_str().map(|s| !s.is_empty()).unwrap_or(true))
                .unwrap_or(false);
            let value = if secret {
                None
            } else {
                current.get(&name).cloned()
            };
            let default = mapping_value(item, "default").map(yaml_json);
            let mut options = Vec::new();
            if let Some(map) = mapping_value(item, "options").and_then(Value::as_mapping) {
                for (k, v) in map {
                    let key = yaml_scalar(Some(k)).unwrap_or_default();
                    let label = yaml_scalar(Some(v)).unwrap_or_else(|| key.clone());
                    options.push((key, label));
                }
            }
            settings_out.push(IndexerSetting {
                name,
                label,
                field_type,
                secret,
                configured,
                value,
                default,
                options,
            });
        }
    }
    // Cardigann defaults username/password when private and no explicit settings
    if settings_out.is_empty() && indexer_type.as_deref() == Some("private") {
        for (name, label, field_type) in [
            ("username", "Username", "text"),
            ("password", "Password", "password"),
        ] {
            let configured = current
                .get(name)
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            settings_out.push(IndexerSetting {
                name: name.into(),
                label: label.into(),
                field_type: field_type.into(),
                secret: field_type == "password",
                configured,
                value: if field_type == "password" {
                    None
                } else {
                    current.get(name).cloned()
                },
                default: None,
                options: vec![],
            });
        }
    }
    // These are Oberiz routing settings rather than fields defined by Cardigann.
    // Keeping them alongside the definition settings means they travel with the
    // indexer configuration without changing the upstream YAML file.
    settings_out.push(IndexerSetting {
        name: "oberiz_priority".into(),
        label: "Search priority".into(),
        field_type: "number".into(),
        secret: false,
        configured: current.contains_key("oberiz_priority"),
        value: current.get("oberiz_priority").cloned(),
        default: Some(serde_json::Value::from(100)),
        options: vec![],
    });
    settings_out.push(IndexerSetting {
        name: "oberiz_tag_name".into(),
        label: "Custom qBittorrent tag (optional)".into(),
        field_type: "text".into(),
        secret: false,
        configured: current
            .get("oberiz_tag_name")
            .and_then(|value| value.as_str())
            .is_some_and(|value| !value.trim().is_empty()),
        value: current.get("oberiz_tag_name").cloned(),
        default: None,
        options: vec![],
    });
    // RSS is an Oberiz capability, not a Cardigann definition field. It remains
    // after routing, so priority is always visible before optional feed details.
    let rss_url = current
        .get("rss_url")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    settings_out.push(IndexerSetting {
        name: "rss_url".into(),
        label: "RSS feed URL (optional)".into(),
        field_type: "text".into(),
        secret: false,
        configured: !rss_url.trim().is_empty(),
        value: current.get("rss_url").cloned(),
        default: None,
        options: vec![],
    });
    Ok(Json(IndexerDetail {
        id,
        name,
        description,
        indexer_type,
        language,
        links,
        enabled,
        settings: settings_out,
    }))
}

pub async fn save_indexer_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ConfigRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing: Option<String> =
        sqlx::query_scalar("SELECT config_json FROM indexer_configs WHERE indexer_id=?")
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut merged = existing
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    for (k, v) in payload.values {
        if v.is_null() {
            continue;
        }
        if v.as_str() == Some("") && is_secret(&k, "password") {
            continue;
        }
        merged.insert(k, v);
    }
    if let Some(tag) = merged
        .get("oberiz_tag_name")
        .and_then(|value| value.as_str())
    {
        let tag = tag.trim();
        if tag.contains(',') || tag.contains('\n') || tag.contains('\r') {
            return Err((
                StatusCode::BAD_REQUEST,
                "The custom qBittorrent tag cannot contain commas or line breaks".into(),
            ));
        }
    }
    if let Some(priority) = merged.get("oberiz_priority") {
        let value = priority
            .as_i64()
            .or_else(|| {
                priority
                    .as_str()
                    .and_then(|value| value.parse::<i64>().ok())
            })
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "Search priority must be a whole number".into(),
                )
            })?;
        if !(0..=10_000).contains(&value) {
            return Err((
                StatusCode::BAD_REQUEST,
                "Search priority must be between 0 and 10000".into(),
            ));
        }
    }
    let json = serde_json::Value::Object(merged).to_string();
    sqlx::query(r#"INSERT INTO indexer_configs(indexer_id,enabled,config_json,updated_at)
        VALUES (?,0,?,CURRENT_TIMESTAMP)
        ON CONFLICT(indexer_id) DO UPDATE SET config_json=excluded.config_json,updated_at=CURRENT_TIMESTAMP"#)
        .bind(&id).bind(json).execute(&state.db).await
        .map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn test_indexer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TestResponse>, (StatusCode, String)> {
    let started = std::time::Instant::now();
    match cardigann::test_indexer(&state, &id).await {
        Ok(message) => {
            persist_runtime(
                &state,
                &id,
                "online",
                &message,
                started.elapsed().as_millis() as i64,
            )
            .await;
            history::record(&state.db, "indexer.test", &id, Some(&message), "info").await;
            Ok(Json(TestResponse {
                status: "ok",
                message,
            }))
        }
        Err(error) => {
            persist_runtime(
                &state,
                &id,
                "offline",
                &error,
                started.elapsed().as_millis() as i64,
            )
            .await;
            history::record(&state.db, "indexer.test_failed", &id, Some(&error), "error").await;
            Err((StatusCode::BAD_GATEWAY, error))
        }
    }
}

async fn load_yaml_by_id(
    state: &AppState,
    id: &str,
) -> Result<(Value, String), (StatusCode, String)> {
    let custom = custom_folder(state).await?;
    let upstream = upstream_folder(state).await?;
    for (folder, source) in [(custom, "custom"), (upstream, "upstream")] {
        if !folder.exists() {
            continue;
        }
        let mut dir = tokio::fs::read_dir(&folder)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        {
            let path = entry.path();
            if !is_yaml(&path) {
                continue;
            }
            let raw = tokio::fs::read_to_string(&path)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let yaml: Value = match serde_yaml::from_str(&raw) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if yaml_scalar(mapping_value(&yaml, "id")).as_deref() == Some(id) {
                return Ok((yaml, source.into()));
            }
        }
    }
    Err((StatusCode::NOT_FOUND, format!("Indexer {id} no encontrado")))
}
fn is_secret(name: &str, field_type: &str) -> bool {
    let n = name.to_lowercase();
    field_type == "password"
        || n.contains("password")
        || n.contains("passkey")
        || n.contains("apikey")
        || n.contains("api_key")
        || n == "cookie"
        || n.contains("token")
}
fn yaml_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Number(n) => n
            .as_i64()
            .map(serde_json::Value::from)
            .or_else(|| n.as_f64().map(serde_json::Value::from))
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Sequence(seq) => serde_json::Value::Array(seq.iter().map(yaml_json).collect()),
        _ => serde_json::Value::Null,
    }
}

async fn scan_all(state: &AppState) -> Result<IndexerListResponse, (StatusCode, String)> {
    let upstream = upstream_folder(state).await?;
    let custom = custom_folder(state).await?;

    let config_rows: Vec<(String, bool, String)> =
        sqlx::query_as("SELECT indexer_id, enabled, config_json FROM indexer_configs")
            .fetch_all(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let config_map: std::collections::HashMap<String, (bool, i64)> = config_rows
        .into_iter()
        .map(|(id, enabled, config)| {
            let priority = serde_json::from_str::<serde_json::Value>(&config)
                .ok()
                .and_then(|value| value.get("oberiz_priority").cloned())
                .and_then(|value| {
                    value
                        .as_i64()
                        .or_else(|| value.as_str().and_then(|value| value.parse::<i64>().ok()))
                })
                .unwrap_or(100);
            (id, (enabled, priority))
        })
        .collect();

    #[allow(clippy::type_complexity)]
    let runtime_rows: Vec<(String,String,Option<String>,Option<i64>,Option<String>)> = sqlx::query_as(
        "SELECT indexer_id,last_status,last_message,last_latency_ms,last_checked_at FROM indexer_runtime"
    ).fetch_all(&state.db).await
      .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let runtime_map: IndexerRuntimeMap = runtime_rows
        .into_iter()
        .map(|(id, status, msg, latency, checked)| (id, (status, msg, latency, checked)))
        .collect();

    let mut items = Vec::new();
    scan_folder(&upstream, "upstream", &config_map, &runtime_map, &mut items).await?;
    scan_folder(&custom, "custom", &config_map, &runtime_map, &mut items).await?;

    items.sort_by_key(|a| a.name.to_lowercase());

    let upstream_count = items.iter().filter(|x| x.source == "upstream").count();
    let custom_count = items.iter().filter(|x| x.source == "custom").count();
    let enabled_count = items.iter().filter(|x| x.enabled).count();
    let invalid = items.iter().filter(|x| !x.valid).count();

    Ok(IndexerListResponse {
        status: "ok",
        total: items.len(),
        upstream: upstream_count,
        custom: custom_count,
        enabled: enabled_count,
        invalid,
        indexers: items,
    })
}

type IndexerRuntimeMap =
    std::collections::HashMap<String, (String, Option<String>, Option<i64>, Option<String>)>;

async fn scan_folder(
    folder: &FsPath,
    source: &str,
    config_map: &std::collections::HashMap<String, (bool, i64)>,
    runtime_map: &IndexerRuntimeMap,
    out: &mut Vec<IndexerSummary>,
) -> Result<(), (StatusCode, String)> {
    if !folder.exists() {
        return Ok(());
    }

    let mut dir = tokio::fs::read_dir(folder).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("No se pudo leer {}: {e}", folder.display()),
        )
    })?;

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        let path = entry.path();
        if !is_yaml(&path) {
            continue;
        }

        let raw = match tokio::fs::read_to_string(&path).await {
            Ok(v) => v,
            Err(e) => {
                out.push(IndexerSummary {
                    id: file_stem(&path),
                    name: file_stem(&path),
                    description: None,
                    language: None,
                    indexer_type: None,
                    links: vec![],
                    source: source.into(),
                    file_name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into(),
                    valid: false,
                    configured: false,
                    enabled: false,
                    priority: 100,
                    settings_count: 0,
                    last_status: "unknown".into(),
                    last_message: None,
                    last_latency_ms: None,
                    last_checked_at: None,
                    error: Some(e.to_string()),
                });
                continue;
            }
        };

        let yaml: Value = match serde_yaml::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                out.push(IndexerSummary {
                    id: file_stem(&path),
                    name: file_stem(&path),
                    description: None,
                    language: None,
                    indexer_type: None,
                    links: vec![],
                    source: source.into(),
                    file_name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into(),
                    valid: false,
                    configured: false,
                    enabled: false,
                    priority: 100,
                    settings_count: 0,
                    last_status: "unknown".into(),
                    last_message: None,
                    last_latency_ms: None,
                    last_checked_at: None,
                    error: Some(format!("YAML: {e}")),
                });
                continue;
            }
        };

        let id = yaml_string(&yaml, "id").unwrap_or_else(|| file_stem(&path));
        let name = yaml_string(&yaml, "name").unwrap_or_else(|| id.clone());
        let description = yaml_string(&yaml, "description");
        let language = yaml_string(&yaml, "language");
        let indexer_type = yaml_string(&yaml, "type");
        let links = yaml_string_list(&yaml, "links");
        let settings_count = mapping_value(&yaml, "settings")
            .and_then(|v| v.as_sequence())
            .map(|s| s.len())
            .unwrap_or(0);

        let mut missing = Vec::new();
        if mapping_value(&yaml, "caps").is_none() {
            missing.push("caps");
        }
        if mapping_value(&yaml, "search").is_none() {
            missing.push("search");
        }
        if links.is_empty() {
            missing.push("links");
        }
        let valid = missing.is_empty();

        let configured = config_map.contains_key(&id);
        let (enabled, priority) = config_map.get(&id).copied().unwrap_or((false, 100));
        let (last_status, last_message, last_latency_ms, last_checked_at) = runtime_map
            .get(&id)
            .cloned()
            .unwrap_or_else(|| ("unknown".into(), None, None, None));

        out.push(IndexerSummary {
            id,
            name,
            description,
            language,
            indexer_type,
            links,
            source: source.into(),
            file_name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            valid,
            configured,
            enabled,
            priority,
            settings_count,
            last_status,
            last_message,
            last_latency_ms,
            last_checked_at,
            error: if valid {
                None
            } else {
                Some(format!("Falta: {}", missing.join(", ")))
            },
        });
    }

    Ok(())
}

fn extract_v11(bytes: Vec<u8>, target: &FsPath) -> Result<usize, String> {
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader).map_err(|e| e.to_string())?;
    let mut installed = 0usize;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().replace('\\', "/");

        if !name.contains("/definitions/v11/") {
            continue;
        }
        if !(name.ends_with(".yml") || name.ends_with(".yaml")) {
            continue;
        }

        let filename = name.rsplit('/').next().unwrap_or("");
        if filename.is_empty() {
            continue;
        }

        let mut contents = Vec::new();
        entry
            .read_to_end(&mut contents)
            .map_err(|e| e.to_string())?;

        let path = target.join(filename);
        let mut file = File::create(&path).map_err(|e| e.to_string())?;
        file.write_all(&contents).map_err(|e| e.to_string())?;
        installed += 1;
    }

    Ok(installed)
}

async fn upstream_folder(state: &AppState) -> Result<PathBuf, (StatusCode, String)> {
    Ok(PathBuf::from(
        settings::get_value(&state.db, "paths.upstream_indexers")
            .await
            .map_err(database_error)?
            .unwrap_or_else(settings::default_upstream_indexers_path),
    ))
}

async fn custom_folder(state: &AppState) -> Result<PathBuf, (StatusCode, String)> {
    Ok(PathBuf::from(
        settings::get_value(&state.db, "paths.custom_indexers")
            .await
            .map_err(database_error)?
            .unwrap_or_else(settings::default_custom_indexers_path),
    ))
}

fn yaml_scalar(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}
fn yaml_list_value(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_sequence)
        .map(|seq| seq.iter().filter_map(|v| yaml_scalar(Some(v))).collect())
        .unwrap_or_default()
}

fn yaml_string(root: &Value, key: &str) -> Option<String> {
    mapping_value(root, key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}
fn yaml_string_list(root: &Value, key: &str) -> Vec<String> {
    mapping_value(root, key)
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}
fn mapping_value<'a>(root: &'a Value, key: &str) -> Option<&'a Value> {
    root.as_mapping()?.get(Value::String(key.to_string()))
}
fn file_stem(path: &FsPath) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into()
}
fn is_yaml(path: &FsPath) -> bool {
    path.extension()
        .and_then(|x| x.to_str())
        .map(|x| x.eq_ignore_ascii_case("yml") || x.eq_ignore_ascii_case("yaml"))
        .unwrap_or(false)
}
fn database_error(error: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
