use crate::settings;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathMapping {
    pub remote: String,
    pub local: String,
}

pub fn path_key(path: &str) -> String {
    let value = path.replace('\\', "/").trim_end_matches('/').to_string();
    if cfg!(windows) {
        value.to_lowercase()
    } else {
        value
    }
}

pub fn contains_path(root: &str, path: &str) -> bool {
    let root = path_key(root);
    let path = path_key(path);
    path == root || path.starts_with(&(root + "/"))
}

pub fn normalize_roots(roots: Vec<String>) -> Result<Vec<String>, (StatusCode, String)> {
    if roots.len() > 64 {
        return Err((StatusCode::BAD_REQUEST, "Too many library folders".into()));
    }
    let mut normalized = Vec::new();
    for root in roots {
        let root = root.trim();
        if root.is_empty() {
            continue;
        }
        let root = crate::filesystem::normalize_display_path(root);
        let path = Path::new(&root);
        if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err((
                StatusCode::BAD_REQUEST,
                "Library folders must be absolute paths without parent traversal".into(),
            ));
        }
        let path = path
            .components()
            .collect::<PathBuf>()
            .to_string_lossy()
            .into_owned();
        if !normalized
            .iter()
            .any(|existing: &String| path_key(existing) == path_key(&path))
        {
            normalized.push(path);
        }
    }
    Ok(normalized)
}

pub async fn roots(db: &SqlitePool, kind: &str) -> Result<Vec<String>, sqlx::Error> {
    let key = if kind == "movie" {
        "library.movie_roots"
    } else {
        "library.series_roots"
    };
    if let Some(value) = settings::get_value(db, key).await? {
        return serde_json::from_str(&value).map_err(|e| sqlx::Error::Protocol(e.to_string()));
    }
    // Legacy import destinations remain valid discovery roots until the user edits the list.
    let legacy = if kind == "movie" {
        "paths.movies"
    } else {
        "paths.series"
    };
    Ok(settings::get_value(db, legacy)
        .await?
        .filter(|p| !p.trim().is_empty())
        .into_iter()
        .collect())
}

pub async fn mappings(db: &SqlitePool) -> Result<Vec<PathMapping>, sqlx::Error> {
    let mut mappings: Vec<PathMapping> =
        match settings::get_value(db, "qbittorrent.path_mappings").await? {
            Some(value) => {
                serde_json::from_str(&value).map_err(|e| sqlx::Error::Protocol(e.to_string()))?
            }
            None => Vec::new(),
        };
    let remote = settings::get_value(db, "qbittorrent.remote_path")
        .await?
        .unwrap_or_default();
    let local = settings::get_value(db, "qbittorrent.local_path")
        .await?
        .unwrap_or_default();
    if !remote.trim().is_empty()
        && !local.trim().is_empty()
        && !mappings
            .iter()
            .any(|m| m.remote == remote && m.local == local)
    {
        mappings.push(PathMapping { remote, local });
    }
    mappings.sort_by_key(|m| std::cmp::Reverse(m.remote.len()));
    Ok(mappings)
}

pub fn map_path(path: &str, mappings: &[PathMapping]) -> String {
    for mapping in mappings {
        if contains_path(&mapping.remote, path) {
            let remote = mapping.remote.trim_end_matches(['/', '\\']);
            let local = mapping.local.trim_end_matches(['/', '\\']);
            let offset = path
                .char_indices()
                .nth(remote.chars().count())
                .map(|(i, _)| i)
                .unwrap_or(path.len());
            return format!("{local}{}", &path[offset..])
                .replace(['/', '\\'], std::path::MAIN_SEPARATOR_STR);
        }
    }
    path.to_string()
}

pub fn normalize_mappings(mappings: Vec<PathMapping>) -> Result<Vec<PathMapping>, StatusCode> {
    if mappings.len() > 64 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut output = Vec::new();
    for mapping in mappings {
        // An unused UI row is not a mapping. Partly filled rows still fail.
        if mapping.remote.trim().is_empty() && mapping.local.trim().is_empty() {
            continue;
        }
        let remote = mapping.remote.trim().to_string();
        let path = remote.replace('\\', "/");
        let remote_absolute = path.starts_with('/')
            || (path.len() > 2
                && path.as_bytes()[0].is_ascii_alphabetic()
                && path.as_bytes()[1] == b':'
                && path.as_bytes()[2] == b'/');
        if !remote_absolute || path.split('/').any(|part| part == "..") {
            return Err(StatusCode::BAD_REQUEST);
        }
        let local = normalize_roots(vec![mapping.local])
            .map_err(|e| e.0)?
            .into_iter()
            .next()
            .ok_or(StatusCode::BAD_REQUEST)?;
        if output
            .iter()
            .any(|m: &PathMapping| path_key(&m.remote) == path_key(&remote))
        {
            return Err(StatusCode::BAD_REQUEST);
        }
        output.push(PathMapping { remote, local });
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_mapping_rows_are_ignored_but_incomplete_rows_are_rejected() {
        assert!(
            normalize_mappings(vec![PathMapping {
                remote: " ".into(),
                local: "".into()
            }])
            .unwrap()
            .is_empty()
        );
        assert!(
            normalize_mappings(vec![PathMapping {
                remote: "/downloads".into(),
                local: "".into()
            }])
            .is_err()
        );
    }
    #[test]
    fn path_boundaries_do_not_confuse_disks_or_sibling_folders() {
        assert!(contains_path("/media/disk", "/media/disk/movie.mkv"));
        assert!(!contains_path("/media/disk", "/media/disk2/movie.mkv"));
    }
    #[test]
    fn mapping_uses_a_component_boundary_and_preserves_unrelated_paths() {
        let mappings = vec![PathMapping {
            remote: "/media/drive".into(),
            local: std::env::temp_dir().to_string_lossy().into_owned(),
        }];
        assert_eq!(
            map_path("/media/drive2/a.mkv", &mappings),
            "/media/drive2/a.mkv"
        );
        assert!(map_path("/media/drive/a.mkv", &mappings).ends_with("a.mkv"));
    }
    #[tokio::test]
    async fn explicit_empty_roots_do_not_restore_a_removed_legacy_root() {
        let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE settings(key TEXT PRIMARY KEY,value TEXT,updated_at TEXT)")
            .execute(&db)
            .await
            .unwrap();
        settings::set_value(&db, "paths.movies", "/old")
            .await
            .unwrap();
        assert_eq!(roots(&db, "movie").await.unwrap(), vec!["/old"]);
        settings::set_value(&db, "library.movie_roots", "[]")
            .await
            .unwrap();
        assert!(roots(&db, "movie").await.unwrap().is_empty());
    }
}
