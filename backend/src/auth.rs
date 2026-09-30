//! Admin-password protection for the internal API.
//!
//! Integration endpoints keep using their independent API-key checks. Internal
//! sessions are opaque, randomly generated tokens whose hashes are persisted,
//! allowing a logout or password change to revoke them immediately.
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{
        SaltString,
        rand_core::{OsRng, RngCore},
    },
};
use axum::{
    Json,
    extract::{Request, State, connect_info::ConnectInfo},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::{AppState, settings};

const COOKIE_NAME: &str = "oberiz_session";
const SESSION_TTL_SECS: i64 = 30 * 24 * 3600;
const PASSWORD_HASH_KEY: &str = "auth.password_hash";
const LOGIN_RESET_AFTER: Duration = Duration::from_secs(15 * 60);
const LOGIN_INITIAL_BLOCK: Duration = Duration::from_secs(30);
const LOGIN_MAX_BLOCK: Duration = Duration::from_secs(60 * 60);

#[derive(Clone, Copy)]
struct LoginAttempt {
    failures: u32,
    last_failure: Instant,
    blocked_until: Option<Instant>,
}

static LOGIN_ATTEMPTS: OnceLock<Mutex<HashMap<String, LoginAttempt>>> = OnceLock::new();

fn attempts() -> &'static Mutex<HashMap<String, LoginAttempt>> {
    LOGIN_ATTEMPTS.get_or_init(|| Mutex::new(HashMap::new()))
}
/// IPs allowed to set `X-Forwarded-For`/`X-Real-IP` (a reverse proxy in front
/// of Oberiz), configured via a comma-separated `OBERIZ_TRUSTED_PROXIES` env
/// var. Left empty by default: an untrusted client could otherwise put any
/// value in that header and rate-limit someone else's IP instead of its own.
fn trusted_proxies() -> &'static [IpAddr] {
    static PROXIES: OnceLock<Vec<IpAddr>> = OnceLock::new();
    PROXIES.get_or_init(|| {
        std::env::var("OBERIZ_TRUSTED_PROXIES")
            .ok()
            .map(|value| {
                value
                    .split(',')
                    .filter_map(|part| part.trim().parse::<IpAddr>().ok())
                    .collect()
            })
            .unwrap_or_default()
    })
}
/// The identity used as the login rate-limit key: the direct peer address,
/// unless it is a configured trusted proxy, in which case the client IP it
/// forwarded is used instead so one shared proxy IP can't lock out everyone
/// behind it.
fn client_identity(headers: &HeaderMap, peer: SocketAddr) -> String {
    if trusted_proxies().contains(&peer.ip()) {
        let forwarded = headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next_back())
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(client) = forwarded {
            return client.to_string();
        }
        let real_ip = headers
            .get("x-real-ip")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(client) = real_ip {
            return client.to_string();
        }
    }
    peer.ip().to_string()
}
fn internal<E: std::fmt::Display>(error: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
fn token_hash(token: &str) -> String {
    hex_encode(&Sha256::digest(token.as_bytes()))
}
fn hash_password(password: &str) -> Result<String, (StatusCode, String)> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(internal)
}
fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).ok().is_some_and(|parsed| {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
}
fn extract_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    raw.split(';').find_map(|part| {
        let part = part.trim();
        part.strip_prefix(name)
            .and_then(|rest| rest.strip_prefix('='))
            .map(str::to_owned)
    })
}
fn secure_cookies_enabled() -> bool {
    matches!(
        std::env::var("OBERIZ_COOKIE_SECURE").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}
fn session_cookie_header(value: &str, max_age_secs: i64) -> HeaderValue {
    let secure = if secure_cookies_enabled() {
        "; Secure"
    } else {
        ""
    };
    HeaderValue::from_str(&format!(
        "{COOKIE_NAME}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_secs}{secure}"
    ))
    .unwrap_or_else(|_| HeaderValue::from_static(""))
}
fn rate_limited(client: &str) -> bool {
    let now = Instant::now();
    let mut attempts = attempts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(attempt) = attempts.get_mut(client) else {
        return false;
    };
    if attempt.blocked_until.is_some_and(|until| now < until) {
        return true;
    }
    if now.duration_since(attempt.last_failure) > LOGIN_RESET_AFTER {
        attempts.remove(client);
        return false;
    }
    false
}
fn record_failed_login(client: &str) {
    let now = Instant::now();
    let mut attempts = attempts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let attempt = attempts.entry(client.to_owned()).or_insert(LoginAttempt {
        failures: 0,
        last_failure: now,
        blocked_until: None,
    });
    if now.duration_since(attempt.last_failure) > LOGIN_RESET_AFTER {
        attempt.failures = 0;
        attempt.blocked_until = None;
    }
    attempt.failures += 1;
    attempt.last_failure = now;
    if attempt.failures >= 5 {
        let multiplier = 1u64 << (attempt.failures - 5).min(7);
        let delay = LOGIN_INITIAL_BLOCK
            .saturating_mul(multiplier as u32)
            .min(LOGIN_MAX_BLOCK);
        attempt.blocked_until = Some(now + delay);
    }
}
fn clear_login_attempts(client: &str) {
    attempts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .remove(client);
}

async fn create_session(db: &SqlitePool) -> Result<String, (StatusCode, String)> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token = hex_encode(&bytes);
    sqlx::query("DELETE FROM auth_sessions WHERE expires_at <= ?")
        .bind(now_secs())
        .execute(db)
        .await
        .map_err(internal)?;
    sqlx::query("INSERT INTO auth_sessions(token_hash, expires_at) VALUES(?, ?)")
        .bind(token_hash(&token))
        .bind(now_secs() + SESSION_TTL_SECS)
        .execute(db)
        .await
        .map_err(internal)?;
    Ok(token)
}
async fn valid_session(db: &SqlitePool, headers: &HeaderMap) -> Result<bool, (StatusCode, String)> {
    let Some(token) = extract_cookie(headers, COOKIE_NAME) else {
        return Ok(false);
    };
    let found = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM auth_sessions WHERE token_hash=? AND expires_at > ? LIMIT 1",
    )
    .bind(token_hash(&token))
    .bind(now_secs())
    .fetch_optional(db)
    .await
    .map_err(internal)?;
    Ok(found.is_some())
}
async fn revoke_session(db: &SqlitePool, headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
    if let Some(token) = extract_cookie(headers, COOKIE_NAME) {
        sqlx::query("DELETE FROM auth_sessions WHERE token_hash=?")
            .bind(token_hash(&token))
            .execute(db)
            .await
            .map_err(internal)?;
    }
    Ok(())
}
async fn revoke_all_sessions(db: &SqlitePool) -> Result<(), (StatusCode, String)> {
    sqlx::query("DELETE FROM auth_sessions")
        .execute(db)
        .await
        .map_err(internal)?;
    Ok(())
}

pub async fn is_enabled(db: &SqlitePool) -> Result<bool, sqlx::Error> {
    Ok(settings::get_value(db, PASSWORD_HASH_KEY)
        .await?
        .is_some_and(|value| !value.is_empty()))
}
fn is_public_path(path: &str) -> bool {
    matches!(
        path,
        "/api/health"
            | "/api/health/live"
            | "/api/health/ready"
            | "/api/auth/status"
            | "/api/auth/login"
            | "/api/auth/logout"
    ) || path.starts_with("/api/v1/")
        || path.starts_with("/radarr/")
        || path.starts_with("/sonarr/")
        || !path.starts_with("/api/")
}
fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"error":"authentication_required"})),
    )
        .into_response()
}
fn setup_required_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"error":"setup_required"})),
    )
        .into_response()
}

pub async fn require_auth(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let path = request.uri().path();
    if is_public_path(path) {
        return next.run(request).await;
    }
    let enabled = match is_enabled(&state.db).await {
        Ok(value) => value,
        Err(_) => return unauthorized_response(),
    };
    if !enabled {
        return if path == "/api/auth/password" {
            next.run(request).await
        } else {
            setup_required_response()
        };
    }
    match valid_session(&state.db, request.headers()).await {
        Ok(true) => next.run(request).await,
        Ok(false) | Err(_) => unauthorized_response(),
    }
}

#[derive(Debug, Serialize)]
pub struct AuthStatus {
    pub enabled: bool,
    pub authenticated: bool,
}
pub async fn status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthStatus>, (StatusCode, String)> {
    let enabled = is_enabled(&state.db).await.map_err(internal)?;
    let authenticated = enabled && valid_session(&state.db, &headers).await?;
    Ok(Json(AuthStatus {
        enabled,
        authenticated,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, (StatusCode, String)> {
    let client = client_identity(&headers, address);
    if rate_limited(&client) {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Too many failed sign-in attempts. Try again later.".into(),
        ));
    }
    let hash = settings::get_value(&state.db, PASSWORD_HASH_KEY)
        .await
        .map_err(internal)?
        .unwrap_or_default();
    if hash.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Password protection is not enabled".into(),
        ));
    }
    if !verify_password(&payload.password, &hash) {
        record_failed_login(&client);
        return Err((StatusCode::UNAUTHORIZED, "Incorrect password".into()));
    }
    clear_login_attempts(&client);
    let token = create_session(&state.db).await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        session_cookie_header(&token, SESSION_TTL_SECS),
    );
    Ok((headers, Json(serde_json::json!({"status":"ok"}))).into_response())
}
pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let _ = revoke_session(&state.db, &headers).await;
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::SET_COOKIE, session_cookie_header("", 0));
    (response_headers, StatusCode::NO_CONTENT).into_response()
}

#[derive(Debug, Deserialize)]
pub struct PasswordRequest {
    pub current_password: Option<String>,
    pub new_password: Option<String>,
}
pub async fn set_password(
    State(state): State<AppState>,
    Json(payload): Json<PasswordRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let existing_hash = settings::get_value(&state.db, PASSWORD_HASH_KEY)
        .await
        .map_err(internal)?
        .filter(|value| !value.is_empty());
    if let Some(hash) = &existing_hash
        && !verify_password(&payload.current_password.unwrap_or_default(), hash)
    {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Current password is incorrect".into(),
        ));
    }
    let new_password = payload
        .new_password
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match new_password {
        Some(new_password) => {
            if !(8..=128).contains(&new_password.len()) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Password must be between 8 and 128 characters".into(),
                ));
            }
            settings::set_value(&state.db, PASSWORD_HASH_KEY, &hash_password(new_password)?)
                .await
                .map_err(internal)?;
        }
        None => settings::set_value(&state.db, PASSWORD_HASH_KEY, "")
            .await
            .map_err(internal)?,
    }
    revoke_all_sessions(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
