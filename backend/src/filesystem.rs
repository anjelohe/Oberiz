use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct DirectoryQuery {
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct DirectoryListing {
    pub path: String,
    pub parent: Option<String>,
    pub roots: Vec<String>,
    pub directories: Vec<DirectoryEntry>,
}

/// Lists only immediate child directories for the local Oberiz host.
/// No file contents or file names are exposed through this helper.
pub async fn list_directories(
    State(state): State<crate::AppState>,
    Query(query): Query<DirectoryQuery>,
) -> Result<Json<DirectoryListing>, (StatusCode, String)> {
    if matches!(allowed_roots(), AllowedRoots::Misconfigured) {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "OBERIZ_FS_ALLOWED_ROOTS is set but none of its paths could be resolved; fix the configured roots or the mounts they point to.".into(),
        ));
    }
    let mut available_roots = roots();
    if matches!(allowed_roots(), AllowedRoots::Unset) {
        available_roots.extend(crate::smb::roots(&state).await);
        available_roots.sort();
        available_roots.dedup();
    }
    if query
        .path
        .as_deref()
        .is_none_or(|path| path.trim().is_empty())
    {
        let directories = available_roots
            .iter()
            .map(|root| DirectoryEntry {
                name: root.clone(),
                path: root.clone(),
            })
            .collect();
        return Ok(Json(DirectoryListing {
            path: String::new(),
            parent: None,
            roots: available_roots,
            directories,
        }));
    }
    let requested = query.path.unwrap_or_default();
    let path = PathBuf::from(normalize_display_path(requested.trim()));
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
    // Each entry carries its own full path, resolved server-side, instead of
    // leaving the client to guess a path separator by concatenating the
    // parent path and a name itself — that guess used the browser's own
    // platform (effectively always "\\" in practice) regardless of what the
    // server is actually running on, so clicking into a subfolder on a
    // POSIX host produced a mixed path like "/media\\movies".
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
        .map(|entry| DirectoryEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: display_path(&entry.path()),
        })
        .collect::<Vec<_>>();
    directories.sort_by_key(|entry| entry.name.to_lowercase());
    Ok(Json(DirectoryListing {
        path: display_path(&canonical),
        parent: canonical.parent().map(display_path),
        roots: available_roots,
        directories,
    }))
}

fn display_path(path: &std::path::Path) -> String {
    normalize_display_path(&path.display().to_string())
}

pub(crate) fn normalize_display_path(path: &str) -> String {
    // Removing only \\?\ from a network path produces the relative
    // "UNC\server\share" rather than a usable Windows UNC path.
    if let Some(rest) = path.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{rest}");
    }
    if let Some(rest) = path.strip_prefix(r"\\?\") {
        return rest.to_string();
    }
    // Repair values emitted by the old picker when a Windows draft is saved.
    if cfg!(windows)
        && let Some(rest) = path.strip_prefix(r"UNC\")
    {
        return format!(r"\\{rest}");
    }
    path.to_string()
}

/// `OBERIZ_FS_ALLOWED_ROOTS`: an optional comma-separated allow-list of base
/// directories (e.g. `/media,/downloads,/config` in Docker). Unset by
/// default, so a native install can still browse the whole machine to pick
/// any folder — this is meant for container/appliance deployments that want
/// to fence the picker in to the paths they actually mounted.
enum AllowedRoots {
    /// The variable isn't set at all: no restriction, by design.
    Unset,
    Configured(Vec<PathBuf>),
    /// The variable is set, but every entry failed to canonicalize (a typo,
    /// a mount that hasn't come up yet, a deleted directory). Collapsing
    /// this into "no restriction" would silently drop an administrator's
    /// explicit fence and expose the whole host — this must fail closed
    /// instead, denying everything until the configuration is fixed.
    Misconfigured,
}

fn allowed_roots() -> AllowedRoots {
    let Ok(raw) = std::env::var("OBERIZ_FS_ALLOWED_ROOTS") else {
        return AllowedRoots::Unset;
    };
    let roots = raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter_map(|value| std::fs::canonicalize(value).ok())
        .collect::<Vec<_>>();
    if roots.is_empty() {
        AllowedRoots::Misconfigured
    } else {
        AllowedRoots::Configured(roots)
    }
}

fn within_allowed_roots(path: &std::path::Path) -> bool {
    match allowed_roots() {
        AllowedRoots::Unset => true,
        AllowedRoots::Configured(roots) => roots.iter().any(|root| path.starts_with(root)),
        AllowedRoots::Misconfigured => false,
    }
}

fn roots() -> Vec<String> {
    if let AllowedRoots::Configured(configured) = allowed_roots() {
        return configured
            .into_iter()
            .map(|root| display_path(&root))
            .collect();
    }
    #[cfg(windows)]
    {
        let drives = unsafe { windows_sys::Win32::Storage::FileSystem::GetLogicalDrives() };
        (b'A'..=b'Z')
            .filter_map(|letter| {
                let root = format!("{}:\\", letter as char);
                (drives & (1 << (letter - b'A')) != 0).then_some(root)
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec!["/".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn network_picker_paths_keep_the_unc_prefix() {
        assert_eq!(
            normalize_display_path(r"\\?\UNC\192.168.12.200\wd19tb\peliculas"),
            r"\\192.168.12.200\wd19tb\peliculas"
        );
        assert_eq!(normalize_display_path(r"\\?\C:\Movies"), r"C:\Movies");
        assert_eq!(
            normalize_display_path(r"\\server\share\Movies"),
            r"\\server\share\Movies"
        );
        assert_eq!(normalize_display_path("/media/movies"), "/media/movies");
    }
    #[cfg(windows)]
    #[test]
    fn old_picker_drafts_can_be_saved_as_absolute_network_roots() {
        let paths = crate::library_roots::normalize_roots(vec![
            r"UNC\192.168.12.200\wd19tb\peliculas".into(),
        ])
        .unwrap();
        assert_eq!(paths, vec![r"\\192.168.12.200\wd19tb\peliculas"]);
    }
}
