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

fn roots() -> Vec<String> {
    #[cfg(windows)]
    {
        return (b'A'..=b'Z')
            .filter_map(|letter| {
                let root = format!("{}:\\", letter as char);
                PathBuf::from(&root).is_dir().then_some(root)
            })
            .collect();
    }
    #[cfg(not(windows))]
    {
        vec!["/".to_string()]
    }
}

fn default_directory() -> String {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .map(|dir| dir.display().to_string())
                .unwrap_or_else(|_| ".".into())
        })
}
