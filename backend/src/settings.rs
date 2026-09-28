use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub tmdb_api_key_set: bool,
    pub qbittorrent_host: String,
    pub qbittorrent_port: u16,
    pub qbittorrent_username: String,
    pub qbittorrent_password_set: bool,
    pub qbittorrent_https: bool,
    pub movies_path: String,
    pub series_path: String,
    pub downloads_path: String,
    pub reseed_path: String,
    pub custom_indexers_path: String,
    pub upstream_indexers_path: String,
    pub automation_enabled: bool,
    pub automation_interval_minutes: u64,
    pub rss_enabled: bool,
    pub rss_interval_minutes: u64,
    pub backup_enabled: bool,
    pub backup_interval_hours: u64,
    pub backup_retention_count: u64,

    pub import_enabled: bool,
    pub import_method: String,
    pub rename_enabled: bool,
    pub movie_naming_template: String,
    pub series_naming_template: String,
    pub keep_reseed_metadata: bool,
    pub cleanup_after_seed: bool,
    pub torrent_metadata_path: String,

    pub ui_theme: String,
    pub api_enabled: bool,
    pub api_key_set: bool,
    pub api_key: String,
    pub overseerr_compat_enabled: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateSettingsRequest {
    pub tmdb_api_key: Option<String>,
    pub qbittorrent_host: Option<String>,
    pub qbittorrent_port: Option<u16>,
    pub qbittorrent_username: Option<String>,
    pub qbittorrent_password: Option<String>,
    pub qbittorrent_https: Option<bool>,
    pub movies_path: Option<String>,
    pub series_path: Option<String>,
    pub downloads_path: Option<String>,
    pub reseed_path: Option<String>,
    pub custom_indexers_path: Option<String>,
    pub upstream_indexers_path: Option<String>,
    pub automation_enabled: Option<bool>,
    pub automation_interval_minutes: Option<u64>,
    pub rss_enabled: Option<bool>,
    pub rss_interval_minutes: Option<u64>,
    pub backup_enabled: Option<bool>,
    pub backup_interval_hours: Option<u64>,
    pub backup_retention_count: Option<u64>,

    pub import_enabled: Option<bool>,
    pub import_method: Option<String>,
    pub rename_enabled: Option<bool>,
    pub movie_naming_template: Option<String>,
    pub series_naming_template: Option<String>,
    pub keep_reseed_metadata: Option<bool>,
    pub cleanup_after_seed: Option<bool>,
    pub torrent_metadata_path: Option<String>,

    pub ui_theme: Option<String>,
    pub api_enabled: Option<bool>,
    pub api_key: Option<String>,
    pub overseerr_compat_enabled: Option<bool>,
}

pub(crate) async fn get_value(
    db: &sqlx::SqlitePool,
    key: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(db)
        .await
}

pub(crate) async fn set_value(
    db: &sqlx::SqlitePool,
    key: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES (?, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(key)
        DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(key)
    .bind(value)
    .execute(db)
    .await?;
    Ok(())
}

pub(crate) fn config_directory() -> PathBuf {
    std::env::var_os("OBERIZ_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|directory| directory.join("config"))
                .unwrap_or_else(|_| PathBuf::from("config"))
        })
}

fn absolute_directory(path: String) -> String {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path.to_string_lossy().into_owned()
    } else {
        std::env::current_dir()
            .map(|directory| directory.join(&path).to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string_lossy().into_owned())
    }
}

pub(crate) fn default_custom_indexers_path() -> String {
    config_directory()
        .join("indexers")
        .join("custom")
        .to_string_lossy()
        .into_owned()
}

pub(crate) fn default_upstream_indexers_path() -> String {
    config_directory()
        .join("indexers")
        .join("upstream")
        .to_string_lossy()
        .into_owned()
}

pub(crate) fn default_torrent_metadata_path() -> String {
    std::env::var_os("OBERIZ_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
        .join("torrents")
        .to_string_lossy()
        .into_owned()
}

async fn text(db: &sqlx::SqlitePool, key: &str, default: &str) -> Result<String, StatusCode> {
    Ok(get_value(db, key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .unwrap_or_else(|| default.to_string()))
}
async fn boolean(db: &sqlx::SqlitePool, key: &str, default: bool) -> Result<bool, StatusCode> {
    Ok(get_value(db, key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|v| v == "true")
        .unwrap_or(default))
}

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<SettingsResponse>, StatusCode> {
    let tmdb_api_key = text(&state.db, "tmdb.api_key", "").await?;
    let qbittorrent_host = text(&state.db, "qbittorrent.host", "127.0.0.1").await?;
    let qbittorrent_port = get_value(&state.db, "qbittorrent.port")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8080);
    let qbittorrent_username = text(&state.db, "qbittorrent.username", "").await?;
    let qbittorrent_password = text(&state.db, "qbittorrent.password", "").await?;
    let qbittorrent_https = boolean(&state.db, "qbittorrent.https", false).await?;

    let movies_path = text(&state.db, "paths.movies", "").await?;
    let series_path = text(&state.db, "paths.series", "").await?;
    let downloads_path = text(&state.db, "paths.downloads", "").await?;
    let reseed_path = text(&state.db, "paths.reseed", "").await?;
    let custom_indexers_default = default_custom_indexers_path();
    let custom_indexers_path = absolute_directory(
        text(&state.db, "paths.custom_indexers", &custom_indexers_default).await?,
    );
    let upstream_indexers_default = default_upstream_indexers_path();
    let upstream_indexers_path = absolute_directory(
        text(
            &state.db,
            "paths.upstream_indexers",
            &upstream_indexers_default,
        )
        .await?,
    );
    std::fs::create_dir_all(&custom_indexers_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    std::fs::create_dir_all(&upstream_indexers_path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let automation_enabled = boolean(&state.db, "automation.enabled", false).await?;
    let automation_interval_minutes = get_value(&state.db, "automation.interval_minutes")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(30);
    let rss_enabled = boolean(&state.db, "rss.enabled", false).await?;
    let rss_interval_minutes = get_value(&state.db, "rss.interval_minutes")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(15);
    let backup_enabled = boolean(&state.db, "backup.enabled", false).await?;
    let backup_interval_hours = get_value(&state.db, "backup.interval_hours")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .and_then(|v| v.parse().ok())
        .unwrap_or(24);
    let backup_retention_count = get_value(&state.db, "backup.retention_count")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);

    let import_enabled = boolean(&state.db, "import.enabled", true).await?;
    let import_method = text(&state.db, "import.method", "auto").await?;
    let rename_enabled = boolean(&state.db, "import.rename_enabled", true).await?;
    let movie_naming_template = text(
        &state.db,
        "import.movie_template",
        "{Title} ({Year}) - {Resolution} {Source} {Codec}",
    )
    .await?;
    let series_naming_template = text(
        &state.db,
        "import.series_template",
        "{Title} - S{Season:00}E{Episode:00} - {EpisodeTitle}",
    )
    .await?;
    let keep_reseed_metadata = boolean(&state.db, "import.keep_reseed_metadata", true).await?;
    let cleanup_after_seed = boolean(&state.db, "import.cleanup_after_seed", true).await?;
    let torrent_metadata_default = default_torrent_metadata_path();
    let torrent_metadata_path = text(
        &state.db,
        "import.torrent_metadata_path",
        &torrent_metadata_default,
    )
    .await?;

    let ui_theme = match text(&state.db, "ui.theme", "dark").await?.as_str() {
        "light" => "light".to_string(),
        "middle" => "middle".to_string(),
        _ => "dark".to_string(),
    };
    let api_enabled = boolean(&state.db, "api.enabled", false).await?;
    let api_key = text(&state.db, "api.key", "").await?;
    let overseerr_compat_enabled = boolean(&state.db, "overseerr.compat_enabled", false).await?;

    Ok(Json(SettingsResponse {
        tmdb_api_key_set: !tmdb_api_key.is_empty(),
        qbittorrent_host,
        qbittorrent_port,
        qbittorrent_username,
        qbittorrent_password_set: !qbittorrent_password.is_empty(),
        qbittorrent_https,
        movies_path,
        series_path,
        downloads_path,
        reseed_path,
        custom_indexers_path,
        upstream_indexers_path,
        automation_enabled,
        automation_interval_minutes,
        rss_enabled,
        rss_interval_minutes,
        backup_enabled,
        backup_interval_hours,
        backup_retention_count,
        import_enabled,
        import_method,
        rename_enabled,
        movie_naming_template,
        series_naming_template,
        keep_reseed_metadata,
        cleanup_after_seed,
        torrent_metadata_path,
        ui_theme,
        api_enabled,
        api_key_set: !api_key.is_empty(),
        api_key,
        overseerr_compat_enabled,
    }))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<UpdateSettingsRequest>,
) -> Result<StatusCode, StatusCode> {
    macro_rules! set_text {
        ($field:expr,$key:expr) => {
            if let Some(value) = $field {
                set_value(&state.db, $key, &value)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        };
    }
    macro_rules! set_bool {
        ($field:expr,$key:expr) => {
            if let Some(value) = $field {
                set_value(&state.db, $key, if value { "true" } else { "false" })
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        };
    }

    set_text!(payload.tmdb_api_key, "tmdb.api_key");
    set_text!(payload.qbittorrent_host, "qbittorrent.host");
    if let Some(value) = payload.qbittorrent_port {
        set_value(&state.db, "qbittorrent.port", &value.to_string())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    set_text!(payload.qbittorrent_username, "qbittorrent.username");
    set_text!(payload.qbittorrent_password, "qbittorrent.password");
    set_bool!(payload.qbittorrent_https, "qbittorrent.https");

    set_text!(payload.movies_path, "paths.movies");
    set_text!(payload.series_path, "paths.series");
    set_text!(payload.downloads_path, "paths.downloads");
    set_text!(payload.reseed_path, "paths.reseed");
    set_text!(payload.custom_indexers_path, "paths.custom_indexers");
    set_text!(payload.upstream_indexers_path, "paths.upstream_indexers");

    set_bool!(payload.automation_enabled, "automation.enabled");
    if let Some(value) = payload.automation_interval_minutes {
        set_value(
            &state.db,
            "automation.interval_minutes",
            &value.clamp(5, 1440).to_string(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    set_bool!(payload.rss_enabled, "rss.enabled");
    if let Some(value) = payload.rss_interval_minutes {
        set_value(
            &state.db,
            "rss.interval_minutes",
            &value.clamp(5, 1440).to_string(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    set_bool!(payload.backup_enabled, "backup.enabled");
    if let Some(value) = payload.backup_interval_hours {
        set_value(
            &state.db,
            "backup.interval_hours",
            &value.clamp(1, 720).to_string(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(value) = payload.backup_retention_count {
        set_value(
            &state.db,
            "backup.retention_count",
            &value.clamp(1, 100).to_string(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    set_bool!(payload.import_enabled, "import.enabled");
    if let Some(value) = payload.import_method {
        let normalized = match value.as_str() {
            "hardlink" => "hardlink",
            "copy" => "copy",
            "move" => "move",
            _ => "auto",
        };
        set_value(&state.db, "import.method", normalized)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    set_bool!(payload.rename_enabled, "import.rename_enabled");
    set_text!(payload.movie_naming_template, "import.movie_template");
    set_text!(payload.series_naming_template, "import.series_template");
    set_bool!(payload.keep_reseed_metadata, "import.keep_reseed_metadata");
    set_bool!(payload.cleanup_after_seed, "import.cleanup_after_seed");
    set_text!(
        payload.torrent_metadata_path,
        "import.torrent_metadata_path"
    );

    if let Some(value) = payload.ui_theme {
        let theme = match value.as_str() {
            "light" => "light",
            "middle" => "middle",
            _ => "dark",
        };
        set_value(&state.db, "ui.theme", theme)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    set_bool!(payload.api_enabled, "api.enabled");
    set_text!(payload.api_key, "api.key");
    set_bool!(payload.overseerr_compat_enabled, "overseerr.compat_enabled");

    Ok(StatusCode::NO_CONTENT)
}
