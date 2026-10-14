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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MappingSuggestion {
    pub remote: String,
    pub local: String,
    /// The translated folder exists and is readable from this server.
    pub verified: bool,
    pub example_remote: String,
    pub example_local: String,
}

/// Proposes remote→local mappings for folders qBittorrent reports but this
/// server cannot open as-is (qBittorrent on a NAS, Oberiz on Windows). A
/// local root such as `\\nas\WD19TB` matches the remote path component with
/// the same name: `/volume1/WD19TB/downloads` → `/volume1/WD19TB` ↦
/// `\\nas\WD19TB`. These are only proposals: unverified ones are returned too
/// so the user can see them, but nothing is applied without confirmation.
pub fn suggest_mappings(
    remote_paths: &[String],
    local_roots: &[String],
    existing: &[PathMapping],
    exists: &dyn Fn(&str) -> bool,
) -> Vec<MappingSuggestion> {
    let mut seen_remote = std::collections::HashSet::new();
    let mut out: Vec<MappingSuggestion> = Vec::new();
    for remote in remote_paths {
        let remote = remote.trim();
        if remote.is_empty()
            || !is_absolute_remote(remote)
            || !seen_remote.insert(path_key(remote))
            || existing.iter().any(|m| contains_path(&m.remote, remote))
            || exists(remote)
        {
            continue;
        }
        let separator = if remote.contains('/') { "/" } else { "\\" };
        let parts: Vec<&str> = remote.split(['/', '\\']).collect();
        for local in local_roots {
            let Some(name) = local.split(['/', '\\']).rfind(|part| !part.is_empty()) else {
                continue;
            };
            let Some(index) = parts
                .iter()
                .position(|part| !part.is_empty() && part.eq_ignore_ascii_case(name))
            else {
                continue;
            };
            let prefix = parts[..=index].join(separator);
            let candidate = PathMapping {
                remote: prefix.clone(),
                local: local.clone(),
            };
            if out.iter().any(|s| {
                path_key(&s.remote) == path_key(&prefix) && path_key(&s.local) == path_key(local)
            }) || existing
                .iter()
                .any(|m| path_key(&m.remote) == path_key(&prefix))
            {
                continue;
            }
            let example_local = map_path(remote, std::slice::from_ref(&candidate));
            out.push(MappingSuggestion {
                verified: exists(&example_local),
                remote: prefix,
                local: local.clone(),
                example_remote: remote.to_string(),
                example_local,
            });
        }
    }
    // A broader verified mapping already translates a narrower one the same
    // way (`/v/WD19TB` ↦ `\\nas\WD19TB` covers `/v/WD19TB/movies` ↦
    // `\\nas\WD19TB\movies`); proposing both would only add noise.
    let broad = out.clone();
    out.retain(|narrow| {
        !broad.iter().any(|wide| {
            wide.verified
                && wide.remote.len() < narrow.remote.len()
                && contains_path(&wide.remote, &narrow.remote)
                && path_key(&map_path(
                    &narrow.remote,
                    &[PathMapping {
                        remote: wide.remote.clone(),
                        local: wide.local.clone(),
                    }],
                )) == path_key(&narrow.local)
        })
    });
    out.sort_by(|a, b| {
        b.verified
            .cmp(&a.verified)
            .then(a.remote.len().cmp(&b.remote.len()))
            .then(a.remote.cmp(&b.remote))
    });
    out
}

fn is_absolute_remote(path: &str) -> bool {
    let path = path.replace('\\', "/");
    path.starts_with('/')
        || (path.len() > 2
            && path.as_bytes()[0].is_ascii_alphabetic()
            && path.as_bytes()[1] == b':'
            && path.as_bytes()[2] == b'/')
}

pub async fn mapping_suggestions(
    axum::extract::State(state): axum::extract::State<crate::AppState>,
) -> Result<axum::Json<Vec<MappingSuggestion>>, (StatusCode, String)> {
    let mut remote_paths = Vec::new();
    let mut first_error = None;
    let default_path = match crate::qbittorrent::default_save_path(&state).await {
        Ok(path) => {
            remote_paths.push(path.clone());
            path
        }
        Err(error) => {
            first_error.get_or_insert(error);
            String::new()
        }
    };
    match crate::qbittorrent::list_categories(axum::extract::State(state.clone())).await {
        Ok(axum::Json(list)) => {
            for category in list.categories {
                let path = category.save_path.trim().to_string();
                // A relative category path lives under the default save path.
                if !path.is_empty() && !is_absolute_remote(&path) && !default_path.is_empty() {
                    remote_paths.push(format!(
                        "{}/{path}",
                        default_path.trim_end_matches(['/', '\\'])
                    ));
                } else {
                    remote_paths.push(path);
                }
            }
        }
        Err(error) => {
            first_error.get_or_insert(error);
        }
    }
    if let Ok(torrents) = crate::qbittorrent::list_torrents_internal(&state).await {
        remote_paths.extend(torrents.into_iter().map(|t| t.save_path));
    }
    if remote_paths.iter().all(|p| p.trim().is_empty()) {
        return Err(first_error.unwrap_or((
            StatusCode::BAD_GATEWAY,
            "qBittorrent did not report any download folder".into(),
        )));
    }
    let mut seen = std::collections::HashSet::new();
    remote_paths.retain(|p| !p.trim().is_empty() && seen.insert(path_key(p)));
    remote_paths.truncate(200);

    let mut local_roots = crate::smb::roots(&state).await;
    for kind in ["movie", "series"] {
        local_roots.extend(roots(&state.db, kind).await.unwrap_or_default());
    }
    for key in ["paths.movies", "paths.series", "paths.downloads"] {
        if let Ok(Some(path)) = settings::get_value(&state.db, key).await
            && !path.trim().is_empty()
        {
            local_roots.push(path);
        }
    }
    let existing = mappings(&state.db).await.unwrap_or_default();
    let suggestions = tokio::task::spawn_blocking(move || {
        suggest_mappings(&remote_paths, &local_roots, &existing, &|path: &str| {
            Path::new(path).is_dir()
        })
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(axum::Json(suggestions))
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
    #[test]
    fn suggests_a_share_mapping_for_a_nas_download_folder_and_verifies_it() {
        let exists = |path: &str| path_key(path) == path_key(r"\\nas\WD19TB\downloads");
        let suggestions = suggest_mappings(
            &["/volume1/WD19TB/downloads".into()],
            &[r"\\nas\ST12TB".into(), r"\\nas\WD19TB".into()],
            &[],
            &exists,
        );
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].remote, "/volume1/WD19TB");
        assert_eq!(suggestions[0].local, r"\\nas\WD19TB");
        assert!(suggestions[0].verified);
    }

    #[test]
    fn skips_paths_already_mapped_or_directly_accessible() {
        let mapped = vec![PathMapping {
            remote: "/volume1/WD19TB".into(),
            local: r"\\nas\WD19TB".into(),
        }];
        assert!(
            suggest_mappings(
                &["/volume1/WD19TB/downloads".into()],
                &[r"\\nas\WD19TB".into()],
                &mapped,
                &|_| false
            )
            .is_empty()
        );
        assert!(
            suggest_mappings(
                &[r"D:\Descargas".into()],
                &[r"\\nas\Descargas".into()],
                &[],
                &|path| path == r"D:\Descargas"
            )
            .is_empty()
        );
    }

    #[test]
    fn unverified_proposals_are_kept_but_ranked_last_and_narrow_duplicates_dropped() {
        let exists = |path: &str| path_key(path).starts_with(&path_key(r"\\nas\WD19TB"));
        let suggestions = suggest_mappings(
            &[
                "/volume1/WD19TB/peliculas".into(),
                "/volume1/other/disk".into(),
            ],
            &[
                r"\\nas\WD19TB".into(),
                r"\\nas\WD19TB\peliculas".into(),
                r"\\nas\disk".into(),
            ],
            &[],
            &exists,
        );
        assert_eq!(
            suggestions
                .iter()
                .map(|s| (s.remote.as_str(), s.verified))
                .collect::<Vec<_>>(),
            vec![("/volume1/WD19TB", true), ("/volume1/other/disk", false)]
        );
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
