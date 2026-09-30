use axum::{Json, extract::Query, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct DirectoryQuery {
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DirectoryListing {
    pub path: String,
    pub parent: Option<String>,
    pub roots: Vec<String>,
    pub directories: Vec<String>,
}

/// Lists only immediate child directories for the local Oberiz host.
/// No file contents or file names are exposed through this helper.
pub async fn list_directories(
    Query(query): Query<DirectoryQuery>,
) -> Result<Json<DirectoryListing>, (StatusCode, String)> {
    let requested = query.path.unwrap_or_else(default_directory);
    let path = PathBuf::from(requested.trim());
    if !path.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Folder does not exist or cannot be opened".into(),
        ));
    }
    let canonical = std::fs::canonicalize(&path).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            format!("Cannot open folder: {error}"),
        )
    })?;
    if !within_allowed_roots(&canonical) {
        return Err((
            StatusCode::FORBIDDEN,
            "That folder is outside the directories Oberiz is allowed to browse".into(),
        ));
    }
    let mut directories = std::fs::read_dir(&canonical)
        .map_err(|error| {
            (
                StatusCode::BAD_REQUEST,
                format!("Cannot read folder: {error}"),
            )
        })?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|kind| kind.is_dir())
                .map(|_| entry)
        })
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    directories.sort_by_key(|name| name.to_lowercase());
    Ok(Json(DirectoryListing {
        path: display_path(&canonical),
        parent: canonical.parent().map(display_path),
        roots: roots(),
        directories,
    }))
}

fn display_path(path: &std::path::Path) -> String {
    path.display()
        .to_string()
        .trim_start_matches(r"\\?\")
        .to_string()
}

/// `OBERIZ_FS_ALLOWED_ROOTS`: an optional comma-separated allow-list of base
/// directories (e.g. `/media,/downloads,/config` in Docker). Unset by
/// default, so a native install can still browse the whole machine to pick
/// any folder — this is meant for container/appliance deployments that want
/// to fence the picker in to the paths they actually mounted.
fn allowed_roots() -> Option<Vec<PathBuf>> {
    let raw = std::env::var("OBERIZ_FS_ALLOWED_ROOTS").ok()?;
    let roots = raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter_map(|value| std::fs::canonicalize(value).ok())
        .collect::<Vec<_>>();
    (!roots.is_empty()).then_some(roots)
}

fn within_allowed_roots(path: &std::path::Path) -> bool {
    match allowed_roots() {
        None => true,
        Some(roots) => roots.iter().any(|root| path.starts_with(root)),
    }
}

fn roots() -> Vec<String> {
    if let Some(configured) = allowed_roots() {
        return configured
            .into_iter()
            .map(|root| display_path(&root))
            .collect();
    }
    #[cfg(windows)]
    {
        (b'A'..=b'Z')
            .filter_map(|letter| {
                let root = format!("{}:\\", letter as char);
                PathBuf::from(&root).is_dir().then_some(root)
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec!["/".to_string()]
    }
}

fn default_directory() -> String {
    if let Some(roots) = allowed_roots()
        && let Some(first) = roots.into_iter().next()
    {
        return first.display().to_string();
    }
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .map(|dir| dir.display().to_string())
                .unwrap_or_else(|_| ".".into())
        })
}
