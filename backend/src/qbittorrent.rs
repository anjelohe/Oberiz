use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};
use tokio::time::sleep;

use crate::{AppState, settings};

#[derive(Debug, Serialize)]
pub struct QBittorrentTestResponse {
    pub status: &'static str,
    pub version: String,
    pub host: String,
    pub latency_ms: u128,
    pub auth_method: &'static str,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QBittorrentTorrent {
    #[serde(default)]
    pub hash: String,
    #[serde(default)]
    pub name: String,

    // Sizes / progress
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub total_size: i64,
    #[serde(default)]
    pub amount_left: i64,
    #[serde(default)]
    pub progress: f64,
    #[serde(default)]
    pub availability: f64,

    // Transfer
    #[serde(default)]
    pub dlspeed: i64,
    #[serde(default)]
    pub upspeed: i64,
    #[serde(default)]
    pub dl_limit: i64,
    #[serde(default)]
    pub up_limit: i64,
    #[serde(default)]
    pub downloaded: i64,
    #[serde(default)]
    pub downloaded_session: i64,
    #[serde(default)]
    pub uploaded: i64,
    #[serde(default)]
    pub uploaded_session: i64,
    #[serde(default)]
    pub eta: i64,
    #[serde(default)]
    pub ratio: f64,
    #[serde(default)]
    pub ratio_limit: f64,

    // Swarm
    #[serde(default)]
    pub num_seeds: i64,
    #[serde(default)]
    pub num_complete: i64,
    #[serde(default)]
    pub num_leechs: i64,
    #[serde(default)]
    pub num_incomplete: i64,

    // State / queue
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub force_start: bool,
    #[serde(default)]
    pub seq_dl: bool,
    #[serde(default)]
    pub f_l_piece_prio: bool,
    #[serde(default)]
    pub super_seeding: bool,

    // Organization / source
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub tracker: String,

    // Paths / identity
    #[serde(default)]
    pub save_path: String,
    #[serde(default)]
    pub content_path: String,
    #[serde(default)]
    pub magnet_uri: String,

    // Times
    #[serde(default)]
    pub added_on: i64,
    #[serde(default)]
    pub completion_on: i64,
    #[serde(default)]
    pub last_activity: i64,
    #[serde(default)]
    pub time_active: i64,
    #[serde(default)]
    pub seeding_time: i64,
}

/// A qBittorrent torrent plus the Oberiz job that downloaded it, if any. Only
/// torrents with a job can be rejected: Oberiz picked those releases itself.
#[derive(Debug, Serialize)]
pub struct DownloadRow {
    #[serde(flatten)]
    pub torrent: QBittorrentTorrent,
    pub oberiz_job_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DownloadListResponse {
    pub status: &'static str,
    pub torrents: Vec<DownloadRow>,
}

#[derive(Debug, Deserialize)]
struct QBittorrentCategoryRaw {
    #[serde(default)]
    name: String,
    #[serde(default, rename = "savePath")]
    save_path: String,
}

#[derive(Debug, Serialize)]
pub struct QBittorrentCategory {
    pub name: String,
    pub save_path: String,
}

#[derive(Debug, Serialize)]
pub struct QBittorrentCategoryListResponse {
    pub status: &'static str,
    pub categories: Vec<QBittorrentCategory>,
}

#[derive(Debug, Serialize)]
pub struct QBittorrentTagListResponse {
    pub status: &'static str,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CategoryPayload {
    pub name: String,
    #[serde(default)]
    pub save_path: String,
}

#[derive(Debug, Deserialize)]
pub struct TagsPayload {
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TorrentOrganizationPayload {
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub previous_tags: String,
}

#[derive(Debug, Deserialize)]
pub struct AddMagnetRequest {
    pub url: String,
    #[serde(default)]
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    #[serde(default)]
    pub delete_files: bool,
}

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub status: &'static str,
}

struct QBittorrentConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    https: bool,
}

impl QBittorrentConfig {
    fn base_url(&self) -> String {
        let protocol = if self.https { "https" } else { "http" };

        format!(
            "{protocol}://{}:{}",
            self.host.trim().trim_end_matches('/'),
            self.port
        )
    }
}

struct AuthenticatedResponse {
    response: reqwest::Response,
    auth_method: &'static str,
}

pub async fn test_connection(
    State(state): State<AppState>,
) -> Result<Json<QBittorrentTestResponse>, (StatusCode, String)> {
    let config = load_config(&state).await?;
    let started = Instant::now();

    let result = authenticated_get(&config, "/api/v2/app/version").await?;

    let version = result.response.text().await.map_err(|error| {
        (
            StatusCode::BAD_GATEWAY,
            format!("No se pudo leer la versión de qBittorrent: {error}"),
        )
    })?;

    Ok(Json(QBittorrentTestResponse {
        status: "ok",
        version: version.trim().to_string(),
        host: format!("{}:{}", config.host, config.port),
        latency_ms: started.elapsed().as_millis(),
        auth_method: result.auth_method,
    }))
}

pub async fn list_downloads(
    State(state): State<AppState>,
) -> Result<Json<DownloadListResponse>, (StatusCode, String)> {
    let mut torrents = list_torrents_internal(&state).await?;
    torrents.sort_by_key(|torrent| std::cmp::Reverse(torrent.added_on));

    let jobs = sqlx::query_as::<_, (String, i64)>(
        "SELECT lower(qb_hash),id FROM download_jobs \
         WHERE qb_hash IS NOT NULL AND qb_hash<>'' AND media_id>0 AND status<>'rejected' \
         ORDER BY id",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .into_iter()
    .collect::<std::collections::HashMap<_, _>>();
    let torrents = torrents
        .into_iter()
        .map(|torrent| DownloadRow {
            oberiz_job_id: jobs.get(&torrent.hash.to_lowercase()).copied(),
            torrent,
        })
        .collect();

    Ok(Json(DownloadListResponse {
        status: "ok",
        torrents,
    }))
}

pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<QBittorrentCategoryListResponse>, (StatusCode, String)> {
    let config = load_config(&state).await?;
    let result = authenticated_get(&config, "/api/v2/torrents/categories").await?;
    let raw = result
        .response
        .json::<std::collections::HashMap<String, QBittorrentCategoryRaw>>()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("No se pudieron interpretar las categorías de qBittorrent: {e}"),
            )
        })?;

    let mut categories = raw
        .into_iter()
        .map(|(key, value)| QBittorrentCategory {
            name: if value.name.trim().is_empty() {
                key
            } else {
                value.name
            },
            save_path: value.save_path,
        })
        .collect::<Vec<_>>();
    categories.sort_by_key(|a| a.name.to_lowercase());
    Ok(Json(QBittorrentCategoryListResponse {
        status: "ok",
        categories,
    }))
}

pub async fn create_category(
    State(state): State<AppState>,
    Json(payload): Json<CategoryPayload>,
) -> Result<Json<ActionResponse>, (StatusCode, String)> {
    let name = clean_category_name(&payload.name)?;
    let config = load_config(&state).await?;
    authenticated_form_post(
        &config,
        "/api/v2/torrents/createCategory",
        &[
            ("category", name),
            ("savePath", payload.save_path.trim().to_string()),
        ],
    )
    .await?;
    Ok(Json(ActionResponse { status: "ok" }))
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<CategoryPayload>,
) -> Result<Json<ActionResponse>, (StatusCode, String)> {
    let category = clean_category_name(&name)?;
    let config = load_config(&state).await?;
    authenticated_form_post(
        &config,
        "/api/v2/torrents/editCategory",
        &[
            ("category", category),
            ("savePath", payload.save_path.trim().to_string()),
        ],
    )
    .await?;
    Ok(Json(ActionResponse { status: "ok" }))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ActionResponse>, (StatusCode, String)> {
    let category = clean_category_name(&name)?;
    let config = load_config(&state).await?;
    authenticated_form_post(
        &config,
        "/api/v2/torrents/removeCategories",
        &[("categories", category)],
    )
    .await?;
    Ok(Json(ActionResponse { status: "ok" }))
}

pub async fn list_tags(
    State(state): State<AppState>,
) -> Result<Json<QBittorrentTagListResponse>, (StatusCode, String)> {
    let config = load_config(&state).await?;
    let result = authenticated_get(&config, "/api/v2/torrents/tags").await?;
    let mut tags = result
        .response
        .json::<Vec<String>>()
        .await
        .map_err(|error| {
            (
                StatusCode::BAD_GATEWAY,
                format!("No se pudieron interpretar las etiquetas de qBittorrent: {error}"),
            )
        })?;
    tags.sort_by_key(|tag| tag.to_lowercase());
    Ok(Json(QBittorrentTagListResponse { status: "ok", tags }))
}

pub async fn create_tags(
    State(state): State<AppState>,
    Json(payload): Json<TagsPayload>,
) -> Result<Json<ActionResponse>, (StatusCode, String)> {
    let tags = clean_tags(payload.tags)?;
    let config = load_config(&state).await?;
    authenticated_form_post(&config, "/api/v2/torrents/createTags", &[("tags", tags)]).await?;
    Ok(Json(ActionResponse { status: "ok" }))
}

pub async fn delete_tags(
    State(state): State<AppState>,
    Json(payload): Json<TagsPayload>,
) -> Result<Json<ActionResponse>, (StatusCode, String)> {
    let tags = clean_tags(payload.tags)?;
    let config = load_config(&state).await?;
    authenticated_form_post(&config, "/api/v2/torrents/deleteTags", &[("tags", tags)]).await?;
    Ok(Json(ActionResponse { status: "ok" }))
}

pub async fn update_torrent_organization(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Json(payload): Json<TorrentOrganizationPayload>,
) -> Result<Json<ActionResponse>, (StatusCode, String)> {
    validate_hash(&hash)?;
    let config = load_config(&state).await?;
    authenticated_form_post(
        &config,
        "/api/v2/torrents/setCategory",
        &[
            ("hashes", hash.clone()),
            ("category", payload.category.trim().to_string()),
        ],
    )
    .await?;
    let previous = payload.previous_tags.trim();
    if !previous.is_empty() {
        authenticated_form_post(
            &config,
            "/api/v2/torrents/removeTags",
            &[("hashes", hash.clone()), ("tags", previous.to_string())],
        )
        .await?;
    }
    let tags = payload.tags.trim();
    if !tags.is_empty() {
        authenticated_form_post(
            &config,
            "/api/v2/torrents/addTags",
            &[("hashes", hash), ("tags", tags.to_string())],
        )
        .await?;
    }
    Ok(Json(ActionResponse { status: "ok" }))
}

pub async fn add_magnet(
    State(state): State<AppState>,
    Json(payload): Json<AddMagnetRequest>,
) -> Result<Json<ActionResponse>, (StatusCode, String)> {
    let config = load_config(&state).await?;
    let magnet = payload.url.trim();

    if !magnet.starts_with("magnet:") {
        return Err((
            StatusCode::BAD_REQUEST,
            "Introduce un enlace magnet válido".to_string(),
        ));
    }

    if let Some(hash) = magnet_hex_hash(magnet)
        && torrent_by_hash(&state, &hash).await?.is_some()
    {
        if !payload.category.trim().is_empty() {
            authenticated_form_post(
                &config,
                "/api/v2/torrents/setCategory",
                &[
                    ("hashes", hash.clone()),
                    ("category", payload.category.trim().to_string()),
                ],
            )
            .await?;
        }
        return Ok(Json(ActionResponse { status: "ok" }));
    }

    let mut fields = vec![("urls", magnet.to_string())];
    if !payload.category.trim().is_empty() {
        fields.push(("category", payload.category.trim().to_string()));
    }
    if let Err(error) = authenticated_multipart_post(&config, "/api/v2/torrents/add", &fields).await
    {
        let duplicate = match magnet_hex_hash(magnet) {
            Some(hash) => torrent_by_hash(&state, &hash).await?.is_some(),
            None => false,
        };
        if !duplicate {
            return Err(error);
        }
    }
    Ok(Json(ActionResponse { status: "ok" }))
}

pub async fn stop_torrent(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<ActionResponse>, (StatusCode, String)> {
    validate_hash(&hash)?;

    let config = load_config(&state).await?;
    authenticated_form_post(&config, "/api/v2/torrents/stop", &[("hashes", hash)]).await?;

    Ok(Json(ActionResponse { status: "ok" }))
}

pub async fn start_torrent(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<ActionResponse>, (StatusCode, String)> {
    validate_hash(&hash)?;

    let config = load_config(&state).await?;
    authenticated_form_post(&config, "/api/v2/torrents/start", &[("hashes", hash)]).await?;

    Ok(Json(ActionResponse { status: "ok" }))
}

pub async fn delete_torrent(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Query(query): Query<DeleteQuery>,
) -> Result<Json<ActionResponse>, (StatusCode, String)> {
    validate_hash(&hash)?;

    let config = load_config(&state).await?;
    authenticated_form_post(
        &config,
        "/api/v2/torrents/delete",
        &[
            ("hashes", hash),
            (
                "deleteFiles",
                if query.delete_files {
                    "true".to_string()
                } else {
                    "false".to_string()
                },
            ),
        ],
    )
    .await?;

    Ok(Json(ActionResponse { status: "ok" }))
}

async fn authenticated_get(
    config: &QBittorrentConfig,
    path: &str,
) -> Result<AuthenticatedResponse, (StatusCode, String)> {
    let base_url = config.base_url();
    let client = build_client()?;
    let url = format!("{base_url}{path}");

    let basic_response = client
        .get(&url)
        .basic_auth(&config.username, Some(&config.password))
        .send()
        .await
        .map_err(|error| connection_error(&base_url, error))?;

    if basic_response.status().is_success() {
        return Ok(AuthenticatedResponse {
            response: basic_response,
            auth_method: "basic",
        });
    }

    if !matches!(
        basic_response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return Err(qb_status_error(basic_response.status(), "acceder a la API"));
    }

    login_cookie(&client, config).await?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| connection_error(&base_url, error))?;

    if !response.status().is_success() {
        return Err(qb_status_error(
            response.status(),
            "acceder a la API tras iniciar sesión",
        ));
    }

    Ok(AuthenticatedResponse {
        response,
        auth_method: "cookie",
    })
}

async fn authenticated_form_post(
    config: &QBittorrentConfig,
    path: &str,
    fields: &[(&str, String)],
) -> Result<(), (StatusCode, String)> {
    let base_url = config.base_url();
    let client = build_client()?;
    let url = format!("{base_url}{path}");

    let basic_response = client
        .post(&url)
        .basic_auth(&config.username, Some(&config.password))
        .form(fields)
        .send()
        .await
        .map_err(|error| connection_error(&base_url, error))?;

    if basic_response.status().is_success() {
        return Ok(());
    }

    if !matches!(
        basic_response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        return Err(qb_status_error(
            basic_response.status(),
            "ejecutar la acción",
        ));
    }

    login_cookie(&client, config).await?;

    let response = client
        .post(&url)
        .form(fields)
        .send()
        .await
        .map_err(|error| connection_error(&base_url, error))?;

    if !response.status().is_success() {
        return Err(qb_status_error(
            response.status(),
            "ejecutar la acción tras iniciar sesión",
        ));
    }

    Ok(())
}

async fn authenticated_multipart_post(
    config: &QBittorrentConfig,
    path: &str,
    fields: &[(&str, String)],
) -> Result<(), (StatusCode, String)> {
    let base_url = config.base_url();
    let client = build_client()?;
    let url = format!("{base_url}{path}");

    let boundary = "----OberizBoundary7MA4YWxkTrZu0gW";
    let body = multipart_body(boundary, fields);

    let basic_response = client
        .post(&url)
        .basic_auth(&config.username, Some(&config.password))
        .header(
            reqwest::header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(body.clone())
        .send()
        .await
        .map_err(|error| connection_error(&base_url, error))?;

    if multipart_add_accepted(basic_response, "añadir el magnet").await? {
        return Ok(());
    }

    login_cookie(&client, config).await?;

    let response = client
        .post(&url)
        .header(
            reqwest::header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(body)
        .send()
        .await
        .map_err(|error| connection_error(&base_url, error))?;

    if !multipart_add_accepted(response, "añadir el magnet tras iniciar sesión").await? {
        return Err((
            StatusCode::UNAUTHORIZED,
            "qBittorrent rechazó el usuario o la contraseña".into(),
        ));
    }

    Ok(())
}

/// qBittorrent can reply with HTTP 200 and the literal body `Fails.` when it
/// rejects a torrent or magnet. HTTP status alone would make Oberiz wait for a
/// torrent that was never created.
async fn multipart_add_accepted(
    response: reqwest::Response,
    action: &str,
) -> Result<bool, (StatusCode, String)> {
    let status = response.status();
    if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
        return Ok(false);
    }
    if !status.is_success() {
        return Err(qb_status_error(status, action));
    }
    let body = response.text().await.map_err(|error| {
        (
            StatusCode::BAD_GATEWAY,
            format!("No se pudo leer la respuesta de qBittorrent: {error}"),
        )
    })?;
    if body.trim().eq_ignore_ascii_case("fails.") {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("qBittorrent rechazó la solicitud al {action} (respuesta: Fails.)"),
        ));
    }
    Ok(true)
}

fn multipart_body(boundary: &str, fields: &[(&str, String)]) -> Vec<u8> {
    let mut body = String::new();

    for (name, value) in fields {
        body.push_str(&format!("--{boundary}\r\n"));
        body.push_str(&format!(
            "Content-Disposition: form-data; name=\"{name}\"\r\n\r\n"
        ));
        body.push_str(value);
        body.push_str("\r\n");
    }

    body.push_str(&format!("--{boundary}--\r\n"));
    body.into_bytes()
}

async fn login_cookie(
    client: &reqwest::Client,
    config: &QBittorrentConfig,
) -> Result<(), (StatusCode, String)> {
    if config.username.trim().is_empty() || config.password.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Faltan las credenciales de qBittorrent".to_string(),
        ));
    }

    let base_url = config.base_url();

    let login_response = client
        .post(format!("{base_url}/api/v2/auth/login"))
        .form(&[
            ("username", config.username.as_str()),
            ("password", config.password.as_str()),
        ])
        .send()
        .await
        .map_err(|error| connection_error(&base_url, error))?;

    if login_response.status().is_success() {
        let body = login_response.text().await.map_err(|error| {
            (
                StatusCode::BAD_GATEWAY,
                format!("No se pudo leer la respuesta de login de qBittorrent: {error}"),
            )
        })?;
        if !body.trim().eq_ignore_ascii_case("fails.") {
            return Ok(());
        }
        return Err((
            StatusCode::UNAUTHORIZED,
            "qBittorrent rechazó el usuario o la contraseña".to_string(),
        ));
    }

    if login_response.status() == StatusCode::FORBIDDEN {
        return Err((
            StatusCode::FORBIDDEN,
            "qBittorrent ha bloqueado temporalmente esta IP por demasiados intentos fallidos"
                .to_string(),
        ));
    }

    Err((
        StatusCode::UNAUTHORIZED,
        "qBittorrent rechazó el usuario o la contraseña".to_string(),
    ))
}

fn build_client() -> Result<reqwest::Client, (StatusCode, String)> {
    reqwest::Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(10))
        .user_agent(concat!("Oberiz/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("No se pudo crear el cliente HTTP: {error}"),
            )
        })
}

fn validate_hash(hash: &str) -> Result<(), (StatusCode, String)> {
    let valid_length = hash.len() == 40 || hash.len() == 64;
    let hexadecimal = hash.chars().all(|c| c.is_ascii_hexdigit());

    if !valid_length || !hexadecimal {
        return Err((
            StatusCode::BAD_REQUEST,
            "Hash de torrent no válido".to_string(),
        ));
    }

    Ok(())
}

fn clean_category_name(value: &str) -> Result<String, (StatusCode, String)> {
    let name = value.trim();
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Category name is required".to_string(),
        ));
    }
    if name.contains('\n') || name.contains('\r') || name.contains(',') {
        return Err((
            StatusCode::BAD_REQUEST,
            "Category names cannot contain commas or line breaks".to_string(),
        ));
    }
    Ok(name.to_string())
}

fn clean_tags(values: Vec<String>) -> Result<String, (StatusCode, String)> {
    let tags = values
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    if tags.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "At least one tag is required".to_string(),
        ));
    }
    if tags
        .iter()
        .any(|tag| tag.contains('\n') || tag.contains('\r') || tag.contains(','))
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Tags cannot contain commas or line breaks".to_string(),
        ));
    }
    Ok(tags.join(","))
}

async fn load_config(state: &AppState) -> Result<QBittorrentConfig, (StatusCode, String)> {
    let host = settings::get_value(&state.db, "qbittorrent.host")
        .await
        .map_err(database_error)?
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let port = settings::get_value(&state.db, "qbittorrent.port")
        .await
        .map_err(database_error)?
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    let username = settings::get_value(&state.db, "qbittorrent.username")
        .await
        .map_err(database_error)?
        .unwrap_or_default();

    let password = settings::get_value(&state.db, "qbittorrent.password")
        .await
        .map_err(database_error)?
        .unwrap_or_default();

    let https = settings::get_value(&state.db, "qbittorrent.https")
        .await
        .map_err(database_error)?
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if host.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "El host de qBittorrent no está configurado".to_string(),
        ));
    }

    Ok(QBittorrentConfig {
        host,
        port,
        username,
        password,
        https,
    })
}

fn qb_status_error(status: StatusCode, action: &str) -> (StatusCode, String) {
    (
        StatusCode::BAD_GATEWAY,
        format!("qBittorrent respondió con HTTP {status} al {action}"),
    )
}

fn connection_error(base_url: &str, error: reqwest::Error) -> (StatusCode, String) {
    (
        StatusCode::BAD_GATEWAY,
        format!("No se pudo conectar con qBittorrent en {base_url}: {error}"),
    )
}

fn database_error(error: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Error leyendo la configuración: {error}"),
    )
}

// Only the tags the user's profile asks for are ever written to qBittorrent:
// no internal marker (e.g. a job id) is added, since those show up in the
// user's own client as clutter next to their real tags. Job recovery after an
// interrupted grab is done by journaling the infohash on the job row *before*
// the hand-off instead (see search_api::grab_internal).
fn oberiz_tags(user_tags: &str, _job_id: Option<i64>, _reseed: bool) -> String {
    let mut tags = Vec::<String>::new();
    for raw in user_tags.split(',') {
        let clean = raw.trim().replace(['\r', '\n'], " ");
        if !clean.is_empty() && !tags.iter().any(|x| x.eq_ignore_ascii_case(&clean)) {
            tags.push(clean);
        }
    }
    tags.join(",")
}

fn normalize_title(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The magnet URI spec (BEP 9) allows a v1 infohash to be either 40 hex
/// characters or, less commonly but just as validly, 32 base32 characters —
/// some indexers emit the base32 form. This used to only recognize hex,
/// silently falling back to less precise duplicate detection for any magnet
/// using base32.
pub(crate) fn magnet_hex_hash(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    for (key, value) in parsed.query_pairs() {
        if key != "xt" {
            continue;
        }
        let value = value.to_string();
        let Some(hash) = value
            .strip_prefix("urn:btih:")
            .or_else(|| value.strip_prefix("URN:BTIH:"))
        else {
            continue;
        };
        if hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(hash.to_ascii_lowercase());
        }
        if hash.len() == 32
            && let Some(hex) = base32_to_hex(hash)
        {
            return Some(hex);
        }
    }
    None
}

/// Decodes a 32-character RFC 4648 base32 string into the 20 raw bytes of a
/// BitTorrent v1 infohash, returned as lowercase hex. BitTorrent v2/hybrid
/// magnets (`urn:btmh:`, a SHA-256 multihash) are a different, larger hash
/// entirely and aren't handled here — qBittorrent's own reported torrent hash
/// for those isn't a same-length infohash to compare against in the first
/// place, so recognizing the URN alone wouldn't be enough to actually match.
fn base32_to_hex(input: &str) -> Option<String> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    if input.len() != 32 {
        return None;
    }
    let mut bits: u64 = 0;
    let mut bit_count: u32 = 0;
    let mut bytes = Vec::with_capacity(20);
    for c in input.to_ascii_uppercase().bytes() {
        let value = ALPHABET.iter().position(|&b| b == c)? as u64;
        bits = (bits << 5) | value;
        bit_count += 5;
        if bit_count >= 8 {
            bit_count -= 8;
            bytes.push((bits >> bit_count) as u8);
        }
    }
    if bytes.len() != 20 {
        return None;
    }
    Some(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

async fn torrent_by_hash(
    state: &AppState,
    hash: &str,
) -> Result<Option<QBittorrentTorrent>, (StatusCode, String)> {
    Ok(list_torrents_fresh(state)
        .await?
        .into_iter()
        .find(|t| t.hash.eq_ignore_ascii_case(hash)))
}

async fn snapshot_hashes(state: &AppState) -> Result<HashSet<String>, (StatusCode, String)> {
    Ok(list_torrents_fresh(state)
        .await?
        .into_iter()
        .map(|t| t.hash)
        .collect())
}

async fn wait_for_new_torrent(
    state: &AppState,
    before: &HashSet<String>,
    title: &str,
    expected_hash: Option<&str>,
) -> Result<QBittorrentTorrent, (StatusCode, String)> {
    let wanted_title = normalize_title(title);
    for _ in 0..40 {
        let torrents = list_torrents_fresh(state).await?;
        if let Some(expected) = expected_hash {
            // A known hash is the only acceptable identity here: if it hasn't
            // appeared yet, keep waiting for it specifically instead of
            // falling through to "the only new torrent" below — someone else
            // adding a different torrent at the same moment would otherwise
            // get mistaken for ours, and its category/tags (and later, its
            // files) would be applied to a download that was never ours.
            if let Some(torrent) = torrents
                .iter()
                .find(|t| t.hash.eq_ignore_ascii_case(expected))
            {
                return Ok(QBittorrentTorrent {
                    hash: torrent.hash.clone(),
                    name: torrent.name.clone(),
                    size: torrent.size,
                    total_size: torrent.total_size,
                    amount_left: torrent.amount_left,
                    progress: torrent.progress,
                    availability: torrent.availability,
                    dlspeed: torrent.dlspeed,
                    upspeed: torrent.upspeed,
                    dl_limit: torrent.dl_limit,
                    up_limit: torrent.up_limit,
                    downloaded: torrent.downloaded,
                    downloaded_session: torrent.downloaded_session,
                    uploaded: torrent.uploaded,
                    uploaded_session: torrent.uploaded_session,
                    eta: torrent.eta,
                    ratio: torrent.ratio,
                    ratio_limit: torrent.ratio_limit,
                    num_seeds: torrent.num_seeds,
                    num_complete: torrent.num_complete,
                    num_leechs: torrent.num_leechs,
                    num_incomplete: torrent.num_incomplete,
                    state: torrent.state.clone(),
                    priority: torrent.priority,
                    force_start: torrent.force_start,
                    seq_dl: torrent.seq_dl,
                    f_l_piece_prio: torrent.f_l_piece_prio,
                    super_seeding: torrent.super_seeding,
                    category: torrent.category.clone(),
                    tags: torrent.tags.clone(),
                    tracker: torrent.tracker.clone(),
                    save_path: torrent.save_path.clone(),
                    content_path: torrent.content_path.clone(),
                    magnet_uri: torrent.magnet_uri.clone(),
                    added_on: torrent.added_on,
                    completion_on: torrent.completion_on,
                    last_activity: torrent.last_activity,
                    time_active: torrent.time_active,
                    seeding_time: torrent.seeding_time,
                });
            }
            sleep(Duration::from_millis(250)).await;
            continue;
        }

        // Only reached when we never had a hash to identify the torrent by
        // (e.g. a magnet link with no btih we could parse) — a best-effort
        // fallback, not used when a specific hash is expected but pending.
        let mut fresh = torrents
            .into_iter()
            .filter(|t| !before.contains(&t.hash))
            .collect::<Vec<_>>();
        if fresh.len() == 1 {
            return Ok(fresh.remove(0));
        }
        if !wanted_title.is_empty()
            && let Some(pos) = fresh.iter().position(|t| {
                let current = normalize_title(&t.name);
                current == wanted_title
                    || current.contains(&wanted_title)
                    || wanted_title.contains(&current)
            })
        {
            return Ok(fresh.remove(pos));
        }
        sleep(Duration::from_millis(250)).await;
    }
    Err((StatusCode::BAD_GATEWAY,
        "qBittorrent añadió la descarga, pero Oberiz no pudo identificar el torrent nuevo para aplicar/validar categoría y etiquetas.".into()))
}

fn tag_set(value: &str) -> HashSet<String> {
    value
        .split(',')
        .map(|x| x.trim().to_lowercase())
        .filter(|x| !x.is_empty())
        .collect()
}

async fn apply_and_verify_metadata(
    state: &AppState,
    hash: &str,
    category: &str,
    tags: &str,
    save_path: Option<&str>,
) -> Result<QBittorrentTorrent, (StatusCode, String)> {
    let config = load_config(state).await?;

    if let Some(save_path) = save_path.filter(|p| !p.trim().is_empty()) {
        // qBittorrent treats adding a torrent whose infohash it already knows
        // as a no-op beyond refreshing category/tags — it keeps seeding from
        // wherever that existing entry's savepath already points. Without
        // this, reseed's freshly reconstructed files at the new job folder
        // would simply never be the files qBittorrent is actually serving
        // from, making the whole reconstruction pointless. setLocation moves
        // it there and recheck makes it verify pieces against what's now on
        // disk at that path instead of trusting stale progress state.
        authenticated_form_post(
            &config,
            "/api/v2/torrents/setLocation",
            &[
                ("hashes", hash.to_string()),
                ("location", save_path.to_string()),
            ],
        )
        .await?;
        authenticated_form_post(
            &config,
            "/api/v2/torrents/recheck",
            &[("hashes", hash.to_string())],
        )
        .await?;
    }

    // qBittorrent's dedicated APIs are the authoritative way to set these after add.
    authenticated_form_post(
        &config,
        "/api/v2/torrents/setCategory",
        &[
            ("hashes", hash.to_string()),
            ("category", category.to_string()),
        ],
    )
    .await?;

    if !tags.trim().is_empty() {
        authenticated_form_post(
            &config,
            "/api/v2/torrents/addTags",
            &[("hashes", hash.to_string()), ("tags", tags.to_string())],
        )
        .await?;
    }

    for _ in 0..20 {
        let torrents = list_torrents_fresh(state).await?;
        if let Some(torrent) = torrents
            .into_iter()
            .find(|t| t.hash.eq_ignore_ascii_case(hash))
        {
            let category_ok = torrent.category.trim() == category.trim();
            let wanted = tag_set(tags);
            let actual = tag_set(&torrent.tags);
            let tags_ok = wanted.iter().all(|tag| actual.contains(tag));
            if category_ok && tags_ok {
                invalidate_torrent_cache(state).await;
                return Ok(torrent);
            }
        }
        sleep(Duration::from_millis(200)).await;
    }

    Err((
        StatusCode::BAD_GATEWAY,
        format!(
            "qBittorrent no confirmó la metadata esperada para {hash}: category='{}', tags='{}'.",
            category, tags
        ),
    ))
}

pub(crate) async fn list_torrents_internal(
    state: &AppState,
) -> Result<Vec<QBittorrentTorrent>, (StatusCode, String)> {
    const CACHE_TTL: Duration = Duration::from_secs(8);
    let mut cache = state.torrent_list_cache.lock().await;
    if let Some(cached) = cache.as_ref()
        && cached.fetched_at.elapsed() < CACHE_TTL
    {
        return Ok(cached.torrents.clone());
    }
    let torrents = list_torrents_fresh(state).await?;
    *cache = Some(crate::TorrentListCache {
        fetched_at: Instant::now(),
        torrents: torrents.clone(),
    });
    Ok(torrents)
}

/// A direct API read for operations that must observe a just-added, changed or
/// deleted torrent. UI polling deliberately goes through the short cache above.
async fn list_torrents_fresh(
    state: &AppState,
) -> Result<Vec<QBittorrentTorrent>, (StatusCode, String)> {
    let config = load_config(state).await?;
    let result = authenticated_get(&config, "/api/v2/torrents/info?filter=all").await?;
    result
        .response
        .json::<Vec<QBittorrentTorrent>>()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("No se pudo interpretar la lista de torrents de qBittorrent: {e}"),
            )
        })
}

pub(crate) async fn invalidate_torrent_cache(state: &AppState) {
    *state.torrent_list_cache.lock().await = None;
}

pub(crate) async fn delete_torrent_internal(
    state: &AppState,
    hash: &str,
    delete_files: bool,
) -> Result<(), (StatusCode, String)> {
    validate_hash(hash)?;
    let config = load_config(state).await?;
    authenticated_form_post(
        &config,
        "/api/v2/torrents/delete",
        &[
            ("hashes", hash.to_string()),
            (
                "deleteFiles",
                if delete_files {
                    "true".into()
                } else {
                    "false".into()
                },
            ),
        ],
    )
    .await
}

pub(crate) async fn export_torrent(
    state: &AppState,
    hash: &str,
) -> Result<Vec<u8>, (StatusCode, String)> {
    validate_hash(hash)?;
    let config = load_config(state).await?;
    let result =
        authenticated_get(&config, &format!("/api/v2/torrents/export?hash={hash}")).await?;
    result
        .response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("No se pudo exportar el .torrent: {e}"),
            )
        })
}

pub(crate) async fn add_url(
    state: &AppState,
    url: &str,
    title: &str,
    category: &str,
    user_tags: &str,
    job_id: Option<i64>,
) -> Result<String, (StatusCode, String)> {
    let expected = magnet_hex_hash(url);
    if let Some(hash) = expected.as_deref()
        && torrent_by_hash(state, hash).await?.is_some()
    {
        let tags = oberiz_tags(user_tags, job_id, false);
        let torrent = apply_and_verify_metadata(state, hash, category, &tags, None).await?;
        return Ok(torrent.hash);
    }
    let before = snapshot_hashes(state).await?;
    let config = load_config(state).await?;
    let tags = oberiz_tags(user_tags, job_id, false);
    let mut fields = vec![("urls", url.to_string())];

    // Send them on add, then explicitly set+verify after qBittorrent creates the torrent.
    fields.push(("category", category.to_string()));
    if !tags.is_empty() {
        fields.push(("tags", tags.clone()));
    }

    if let Err(error) = authenticated_multipart_post(&config, "/api/v2/torrents/add", &fields).await
    {
        let already_present = match expected.as_deref() {
            Some(hash) => torrent_by_hash(state, hash).await?.is_some(),
            None => false,
        };
        if !already_present {
            return Err(error);
        }
    }

    let torrent = wait_for_new_torrent(state, &before, title, expected.as_deref()).await?;
    let torrent = apply_and_verify_metadata(state, &torrent.hash, category, &tags, None).await?;
    Ok(torrent.hash)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn add_torrent_bytes_with_options(
    state: &AppState,
    bytes: Vec<u8>,
    category: &str,
    title: &str,
    user_tags: &str,
    job_id: Option<i64>,
    save_path: Option<&str>,
    reseed: bool,
) -> Result<String, (StatusCode, String)> {
    // qBittorrent answers `Fails.` for an already-known torrent file. Detect
    // v1/hybrid torrents by their metainfo hash first so a duplicate can be
    // treated like the magnet path rather than a failed upload.
    if let Some(info_hash) = crate::cardigann::torrent_v1_info_hash(&bytes)
        && torrent_by_hash(state, &info_hash).await?.is_some()
    {
        let tags = oberiz_tags(user_tags, job_id, reseed);
        let torrent =
            apply_and_verify_metadata(state, &info_hash, category, &tags, save_path).await?;
        return Ok(torrent.hash);
    }

    let before = snapshot_hashes(state).await?;
    let config = load_config(state).await?;
    let base_url = config.base_url();
    let client = build_client()?;
    let url = format!("{base_url}/api/v2/torrents/add");
    let tags = oberiz_tags(user_tags, job_id, reseed);

    #[allow(clippy::too_many_arguments)]
    async fn send(
        client: &reqwest::Client,
        url: &str,
        config: &QBittorrentConfig,
        bytes: Vec<u8>,
        category: &str,
        title: &str,
        tags: &str,
        save_path: Option<&str>,
        basic: bool,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let filename = format!(
            "{}.torrent",
            title
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
                .take(80)
                .collect::<String>()
        );
        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(filename)
            .mime_str("application/x-bittorrent")
            .unwrap();
        let mut form = reqwest::multipart::Form::new().part("torrents", part);
        if !category.trim().is_empty() {
            form = form.text("category", category.to_string());
        }
        if !tags.trim().is_empty() {
            form = form.text("tags", tags.to_string());
        }
        if let Some(save_path) = save_path.filter(|p| !p.trim().is_empty()) {
            form = form.text("savepath", save_path.to_string());
        }
        let mut req = client.post(url).multipart(form);
        if basic {
            req = req.basic_auth(&config.username, Some(&config.password));
        }
        req.send().await
    }

    let response = send(
        &client,
        &url,
        &config,
        bytes.clone(),
        category,
        title,
        &tags,
        save_path,
        true,
    )
    .await
    .map_err(|e| connection_error(&base_url, e))?;

    let added = if multipart_add_accepted(response, "añadir el .torrent").await? {
        true
    } else {
        login_cookie(&client, &config).await?;
        let response = send(
            &client, &url, &config, bytes, category, title, &tags, save_path, false,
        )
        .await
        .map_err(|e| connection_error(&base_url, e))?;
        if !multipart_add_accepted(response, "añadir el .torrent tras iniciar sesión").await? {
            return Err((
                StatusCode::UNAUTHORIZED,
                "qBittorrent rechazó el usuario o la contraseña".into(),
            ));
        }
        true
    };

    if added {
        let torrent = wait_for_new_torrent(state, &before, title, None).await?;
        let torrent =
            apply_and_verify_metadata(state, &torrent.hash, category, &tags, None).await?;
        return Ok(torrent.hash);
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::{base32_to_hex, magnet_hex_hash};

    /// RFC 4648 base32, mirroring `base32_to_hex`'s decode direction, so the
    /// round trip below verifies both against the same reference alphabet
    /// rather than against a single hand-computed example.
    fn hex_to_base32(hex: &str) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let mut bits: u64 = 0;
        let mut bit_count: u32 = 0;
        let mut out = String::new();
        for byte in bytes {
            bits = (bits << 8) | byte as u64;
            bit_count += 8;
            while bit_count >= 5 {
                bit_count -= 5;
                out.push(ALPHABET[((bits >> bit_count) & 0x1f) as usize] as char);
            }
        }
        if bit_count > 0 {
            out.push(ALPHABET[((bits << (5 - bit_count)) & 0x1f) as usize] as char);
        }
        out
    }

    /// 20 bytes hex-encoded programmatically (not hand-typed) so the string
    /// is guaranteed to be exactly 40 hex characters — a real SHA-1 infohash's
    /// length.
    fn sample_infohash_hex() -> String {
        let bytes: [u8; 20] = [
            0xc9, 0xe1, 0x57, 0x63, 0xf7, 0x22, 0xf2, 0x3e, 0x98, 0xa2, 0x9d, 0xec, 0xdf, 0xae,
            0x34, 0x1b, 0x98, 0xd5, 0x30, 0x1f,
        ];
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn base32_round_trips_with_a_known_infohash() {
        let hex = sample_infohash_hex();
        let base32 = hex_to_base32(&hex);
        assert_eq!(base32.len(), 32);
        assert_eq!(base32_to_hex(&base32).as_deref(), Some(hex.as_str()));
    }

    #[test]
    fn magnet_hex_hash_recognizes_both_hex_and_base32_btih() {
        let hex = sample_infohash_hex();
        let base32 = hex_to_base32(&hex);
        let hex_magnet = format!("magnet:?xt=urn:btih:{hex}&dn=Example");
        let base32_magnet = format!("magnet:?xt=urn:btih:{base32}&dn=Example");
        assert_eq!(magnet_hex_hash(&hex_magnet).as_deref(), Some(hex.as_str()));
        assert_eq!(
            magnet_hex_hash(&base32_magnet).as_deref(),
            Some(hex.as_str())
        );
    }
}
