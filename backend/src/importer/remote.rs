//! Metadata and journal live in SQLite (and its normal backups), never on the NAS.
use super::{
    DownloadJob, FileMapping, ReseedResponse, internal, media_identity, naming, setting_bool,
};
use crate::{
    AppState, history, qbittorrent::QBittorrentTorrent, qbittorrent::TorrentFile, releases,
    settings,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
type Error = (StatusCode, String);

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Entry {
    index: i64,
    size: i64,
    original: String,
    target: String,
}

fn conflict(message: &str) -> Error {
    (StatusCode::CONFLICT, message.into())
}

pub(crate) fn valid_relative(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4096
        && !path.starts_with(['/', '\\'])
        && !path.contains(['\\', ':'])
        && !path.chars().any(char::is_control)
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}
fn media(path: &str) -> bool {
    path.rsplit('.').next().is_some_and(|ext| {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "mkv" | "mp4" | "avi" | "m4v" | "mov" | "ts" | "m2ts" | "wmv"
        )
    })
}

fn plan(
    files: &[TorrentFile],
    job: &DownloadJob,
    title: &str,
    year: Option<i32>,
    template: &str,
    rename: bool,
) -> Result<Vec<Entry>, Error> {
    if files.is_empty()
        || files.len() > 100_000
        || files
            .iter()
            .any(|f| !valid_relative(&f.name) || f.size < 0 || f.index < 0)
    {
        return Err(conflict("Invalid or unavailable torrent file list"));
    }
    let videos: Vec<_> = files.iter().filter(|f| media(&f.name)).collect();
    if rename && (videos.is_empty() || (job.media_type == "movie" && videos.len() != 1)) {
        return Err(conflict("Ambiguous video selection; no files renamed"));
    }
    // A sidecar may depend on the original basename. Do not silently orphan it.
    if rename
        && files.iter().any(|f| {
            f.name.rsplit('.').next().is_some_and(|ext| {
                matches!(
                    ext.to_ascii_lowercase().as_str(),
                    "srt" | "ass" | "ssa" | "sub" | "idx" | "vtt" | "sup" | "smi" | "ttml"
                )
            })
        })
    {
        return Err(conflict(
            "External subtitles require manual naming; no files renamed",
        ));
    }
    let mut targets = HashSet::new();
    let mut indexes = HashSet::new();
    let mut entries = Vec::new();
    for file in files {
        if !indexes.insert(file.index) {
            return Err(conflict("Duplicate torrent file index"));
        }
        let mut target = file.name.clone();
        if rename && media(&file.name) {
            let parsed = releases::parse(&format!("{} {}", file.name, job.release_title));
            let rendered = if job.media_type == "movie" {
                naming::render_template(template, title, year, &parsed)
            } else {
                let (season, episode, end) = naming::parse_episode_numbers(&file.name)
                    .or_else(|| {
                        (videos.len() == 1 && !job.is_season_pack)
                            .then(|| {
                                job.season_number
                                    .zip(job.episode_number)
                                    .map(|(s, e)| (s, e, None))
                            })
                            .flatten()
                    })
                    .ok_or_else(|| conflict("Episode identity is ambiguous; no files renamed"))?;
                if season < 0
                    || episode < 1
                    || end.is_some_and(|last| last < episode || last - episode > 50)
                {
                    return Err(conflict("Invalid episode range"));
                }
                naming::render_series_template(
                    template, title, year, &parsed, season, episode, end, "",
                )
            };
            let name = naming::sanitize_name(&rendered)
                .trim_end_matches(['.', ' '])
                .to_string();
            let reserved = name.split('.').next().unwrap_or("").to_ascii_uppercase();
            if name.is_empty()
                || name.len() > 220
                || name.contains(['{', '}'])
                || matches!(
                    reserved.as_str(),
                    "CON"
                        | "PRN"
                        | "AUX"
                        | "NUL"
                        | "COM1"
                        | "COM2"
                        | "COM3"
                        | "LPT1"
                        | "LPT2"
                        | "LPT3"
                )
            {
                return Err(conflict("Naming template produced an invalid filename"));
            }
            let extension = file.name.rsplit('.').next().unwrap();
            let prefix = file
                .name
                .rsplit_once('/')
                .map(|(parent, _)| format!("{parent}/"))
                .unwrap_or_default();
            target = format!("{prefix}{name}.{extension}");
        }
        if !valid_relative(&target) || !targets.insert(target.to_lowercase()) {
            return Err(conflict("Naming collision; no files renamed"));
        }
        // No swaps/cycles or case-only renames: fail before any API mutation.
        if target != file.name
            && files
                .iter()
                .any(|other| other.name.eq_ignore_ascii_case(&target))
        {
            return Err(conflict("Target already exists; no files renamed"));
        }
        entries.push(Entry {
            index: file.index,
            size: file.size,
            original: file.name.clone(),
            target,
        });
    }
    Ok(entries)
}

async fn physical_guard(
    state: &AppState,
    save: &str,
    entry: &Entry,
    recovering: bool,
) -> Result<(), Error> {
    let full = |relative: &str| {
        format!(
            "{}/{}",
            save.replace('\\', "/").trim_end_matches('/'),
            relative
        )
    };
    let old = full(&entry.original);
    let target = full(&entry.target);
    let parent = target
        .rsplit_once('/')
        .ok_or_else(|| conflict("Invalid destination"))?
        .0;
    let listed = state
        .download_client
        .directory_entries(state, parent)
        .await?;
    let contains = |path: &str| {
        listed
            .iter()
            .any(|p| p.replace('\\', "/").eq_ignore_ascii_case(path))
    };
    let target_exists = contains(&target);
    let old_parent = old
        .rsplit_once('/')
        .ok_or_else(|| conflict("Invalid source"))?
        .0;
    let source_exists = if old_parent.eq_ignore_ascii_case(parent) {
        contains(&old)
    } else {
        state
            .download_client
            .directory_entries(state, old_parent)
            .await?
            .iter()
            .any(|p| p.replace('\\', "/").eq_ignore_ascii_case(&old))
    };
    // qBittorrent renameFile can overwrite unrelated physical files even when
    // the target is absent from the torrent's own list. Never rely on HTTP 409.
    if target_exists && (!recovering || source_exists) {
        return Err(conflict(
            "A physical naming destination already exists; no overwrite requested",
        ));
    }
    if !source_exists && !(recovering && target_exists) {
        return Err(conflict("Source file is unavailable; stopped naming"));
    }
    Ok(())
}

async fn apply_plan(
    state: &AppState,
    hash: &str,
    save: &str,
    entries: &[Entry],
    recovering: bool,
) -> Result<(), Error> {
    let mut indexes = HashSet::new();
    let mut targets = HashSet::new();
    if entries.is_empty()
        || entries.len() > 100_000
        || entries.iter().any(|entry| {
            !valid_relative(&entry.original)
                || !valid_relative(&entry.target)
                || entry.index < 0
                || entry.size < 0
                || !indexes.insert(entry.index)
                || !targets.insert(entry.target.to_lowercase())
        })
    {
        return Err(conflict("Invalid archived naming plan"));
    }
    let initial = state.download_client.files(state, hash).await?;
    // Preflight every physical target before the first mutation, then check
    // again immediately before each rename. External writers must still be
    // excluded: the remote API does not expose atomic no-replace semantics.
    for entry in entries {
        if initial
            .iter()
            .any(|file| file.index == entry.index && file.name != entry.target)
        {
            physical_guard(state, save, entry, recovering).await?;
        }
    }
    for entry in entries {
        if !valid_relative(&entry.original) || !valid_relative(&entry.target) {
            return Err(conflict("Invalid archived path"));
        }
        let current = state.download_client.files(state, hash).await?;
        if current.len() != entries.len() {
            return Err(conflict("Torrent file list changed; stopped naming"));
        }
        let file = current
            .iter()
            .find(|f| f.index == entry.index && f.size == entry.size)
            .ok_or_else(|| conflict("Torrent content changed; stopped naming"))?;
        if file.name == entry.target {
            continue;
        } // Crash-safe retry of an already applied step.
        if file.name != entry.original
            || current
                .iter()
                .any(|f| f.name.eq_ignore_ascii_case(&entry.target))
        {
            return Err(conflict("Torrent paths changed or naming target occupied"));
        }
        physical_guard(state, save, entry, recovering).await?;
        state
            .download_client
            .rename_file(state, hash, &entry.original, &entry.target)
            .await?;
        // HTTP 200 is not proof: qBittorrent may accept a nonexistent torrent.
        let mut verified = false;
        for _ in 0..10 {
            let files = state.download_client.files(state, hash).await?;
            if files
                .iter()
                .any(|f| f.index == entry.index && f.size == entry.size && f.name == entry.target)
            {
                verified = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        if !verified {
            return Err(conflict(
                "qBittorrent did not confirm the renamed file; stopped",
            ));
        }
    }
    Ok(())
}

pub(super) async fn complete(
    state: &AppState,
    job: &DownloadJob,
    torrent: &QBittorrentTorrent,
) -> Result<(), Error> {
    let _guard = super::MEDIA_OPERATION_GATE.lock().await;
    let rename = setting_bool(state, "qbittorrent.rename_enabled", false).await?;
    let keep = setting_bool(state, "import.keep_reseed_metadata", true).await?;
    let saved: Option<(String, String, String)> =
        sqlx::query_as("SELECT hash,save_path,mappings_json FROM torrent_archives WHERE job_id=?")
            .bind(job.id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?;
    let entries = if let Some((hash, save, mappings)) = saved {
        if hash != torrent.hash || save != torrent.save_path {
            return Err(conflict(
                "Archived torrent destination changed; manual review required",
            ));
        }
        let entries: Vec<Entry> = serde_json::from_str(&mappings).map_err(internal)?;
        if !rename && entries.iter().any(|e| e.original != e.target) {
            return Err(conflict(
                "Pending naming journal: enable naming to finish or review manually",
            ));
        }
        entries
    } else if rename || keep {
        let files = state.download_client.files(state, &torrent.hash).await?;
        let (title, year) = if rename {
            media_identity(state, &job.media_type, job.media_id).await?
        } else {
            (String::new(), None)
        };
        if rename && job.media_id <= 0 {
            return Err(conflict("Unknown media identity; no files renamed"));
        }
        let (key, default) = if job.media_type == "movie" {
            (
                "import.movie_template",
                "{Title} ({Year}) - {Resolution} {Source} {Codec}",
            )
        } else {
            (
                "import.series_template",
                "{Title} - S{Season:00}E{Episode:00} - {Resolution} {Source} {Codec}",
            )
        };
        let template = settings::get_value(&state.db, key)
            .await
            .map_err(internal)?
            .unwrap_or_else(|| default.into());
        let entries = plan(&files, job, &title, year, &template, rename)?;
        let data = state
            .download_client
            .export_torrent_file(state, &torrent.hash)
            .await?;
        if data.is_empty() || data.len() > 32 * 1024 * 1024 || torrent.save_path.trim().is_empty() {
            return Err(conflict("Cannot archive torrent metadata or destination"));
        }
        crate::cardigann::validate_torrent_metainfo(&data)
            .map_err(|_| conflict("Invalid exported torrent metadata"))?;
        if torrent.hash.len() == 40
            && crate::cardigann::torrent_v1_info_hash(&data).as_deref() != Some(&torrent.hash)
        {
            return Err(conflict(
                "Exported torrent identity does not match; no files renamed",
            ));
        }
        sqlx::query("INSERT INTO torrent_archives(job_id,hash,save_path,torrent_data,mappings_json) VALUES(?,?,?,?,?)")
            .bind(job.id).bind(&torrent.hash).bind(&torrent.save_path).bind(data).bind(serde_json::to_string(&entries).map_err(internal)?)
            .execute(&state.db).await.map_err(internal)?;
        entries
    } else {
        Vec::new()
    };
    if rename {
        apply_plan(state, &torrent.hash, &torrent.save_path, &entries, false).await?;
    }
    let mappings: Vec<FileMapping> = entries
        .iter()
        .map(|e| FileMapping {
            original_rel: e.original.clone(),
            library_rel: e.target.clone(),
        })
        .collect();
    let mut tx = state.db.begin().await.map_err(internal)?;
    sqlx::query("UPDATE torrent_archives SET completed=1 WHERE job_id=?")
        .bind(job.id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    sqlx::query("UPDATE download_jobs SET status='seeding',source_path=?,library_path=?,import_method='qbittorrent',file_mappings_json=?,torrent_metadata_path=?,imported_at=CURRENT_TIMESTAMP,last_error=NULL,updated_at=CURRENT_TIMESTAMP WHERE id=?")
        .bind(&torrent.content_path).bind(&torrent.save_path).bind(serde_json::to_string(&mappings).map_err(internal)?)
        .bind((rename||keep).then(||format!("archive:{}",job.id))).bind(job.id).execute(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    history::record(
        &state.db,
        "download.completed",
        &job.release_title,
        Some(if rename {
            "Naming verified through qBittorrent; recovery metadata archived"
        } else {
            "Completed at qBittorrent destination; no media files changed"
        }),
        "info",
    )
    .await;
    Ok(())
}

pub(super) async fn restore(state: &AppState, job: &DownloadJob) -> Result<ReseedResponse, Error> {
    let archive: Option<(String,String,Vec<u8>,String)> = sqlx::query_as("SELECT hash,save_path,torrent_data,mappings_json FROM torrent_archives WHERE job_id=? AND completed=1")
        .bind(job.id).fetch_optional(&state.db).await.map_err(internal)?;
    let (hash, save, data, json) = archive.ok_or_else(|| {
        conflict(
            "No remote recovery archive. Legacy imports require manual recovery in qBittorrent",
        )
    })?;
    if state
        .download_client
        .list_torrents(state)
        .await?
        .iter()
        .any(|t| t.hash == hash)
    {
        return Err(conflict(
            "Torrent already exists in qBittorrent; nothing changed",
        ));
    }
    // v1/hybrid metadata can be checked before adding. Pure v2 needs a separate identity implementation.
    if crate::cardigann::torrent_v1_info_hash(&data).as_deref() != Some(&hash) {
        return Err(conflict(
            "Recovery currently requires verified v1/hybrid torrent metadata",
        ));
    }
    let entries: Vec<Entry> = serde_json::from_str(&json).map_err(internal)?;
    if entries.is_empty()
        || entries
            .iter()
            .any(|e| !valid_relative(&e.original) || !valid_relative(&e.target))
    {
        return Err(conflict("Invalid recovery journal"));
    }
    let restored = state
        .download_client
        .add_torrent_file(
            state,
            data,
            &job.category,
            &job.release_title,
            &job.qbittorrent_tags,
            Some(job.id),
            Some(&save),
            true,
        )
        .await?;
    if restored != hash {
        return Err(conflict(
            "Unexpected restored torrent identity; manual review required",
        ));
    }
    let files = state.download_client.files(state, &hash).await?;
    let recovery: Vec<Entry> = entries
        .iter()
        .map(|entry| {
            let current = files
                .iter()
                .find(|f| f.index == entry.index && f.size == entry.size)
                .ok_or_else(|| conflict("Restored torrent content does not match its archive"))?;
            Ok(Entry {
                original: current.name.clone(),
                ..entry.clone()
            })
        })
        .collect::<Result<_, Error>>()?;
    apply_plan(state, &hash, &save, &recovery, true).await?;
    state.download_client.recheck(state, &hash).await?;
    sqlx::query("UPDATE download_jobs SET status='restored_stopped',cleaned_at=CURRENT_TIMESTAMP,reseed_count=reseed_count+1,last_error=NULL,updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(job.id).execute(&state.db).await.map_err(internal)?;
    history::record(
        &state.db,
        "torrent.restored",
        &job.release_title,
        Some("Restored stopped; qBittorrent recheck requested. Inspect and resume manually."),
        "info",
    )
    .await;
    Ok(ReseedResponse {
        status: "stopped_recheck_requested",
        path: save,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::download_client::DownloadClient;
    use async_trait::async_trait;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };
    use tokio::sync::Mutex;

    const META: &[u8] = b"d4:infod6:lengthi9e4:name7:old.mkv12:piece lengthi16384e6:pieces20:aaaaaaaaaaaaaaaaaaaaee";

    // Invoked only by scripts/test-qbittorrent-isolated.mjs after creating a
    // fresh, verified profile with synthetic files. Never targets user settings.
    #[tokio::test]
    #[ignore = "requires the explicit isolated synthetic qBittorrent probe"]
    async fn real_qbt_conflict_guard() {
        let port =
            std::env::var("OBERIZ_SYNTHETIC_QB_PORT").expect("Run the isolated probe script");
        let directory = std::env::var("OBERIZ_SYNTHETIC_QB_DIRECTORY").unwrap();
        let normalized = directory.replace('\\', "/");
        assert!(
            normalized.contains("/backend/backups/storage-simplification-20261015/real-qbt-")
                && normalized.ends_with("/synthetic-media")
        );
        let (state, _, _) = fixture().await;
        let mut state = state;
        state.download_client = Arc::new(crate::download_client::QBittorrentClient);
        settings::set_value(&state.db, "qbittorrent.host", "127.0.0.1")
            .await
            .unwrap();
        settings::set_value(&state.db, "qbittorrent.port", &port)
            .await
            .unwrap();
        let torrents = state.download_client.list_torrents(&state).await.unwrap();
        assert_eq!(torrents.len(), 1);
        let torrent = &torrents[0];
        assert_eq!(
            torrent.save_path.replace('\\', "/").trim_end_matches('/'),
            normalized
        );
        let files = state
            .download_client
            .files(&state, &torrent.hash)
            .await
            .unwrap();
        assert_eq!(files.len(), 1);
        let entry = Entry {
            index: files[0].index,
            size: files[0].size,
            original: files[0].name.clone(),
            target: "Occupied.mkv".into(),
        };
        let error = apply_plan(&state, &torrent.hash, &directory, &[entry], false)
            .await
            .unwrap_err();
        assert_eq!(error.0, StatusCode::CONFLICT);
        assert!(error.1.contains("physical naming destination"));
        assert_eq!(
            state
                .download_client
                .files(&state, &torrent.hash)
                .await
                .unwrap()[0]
                .name,
            files[0].name
        );
    }
    struct Client {
        files: Mutex<Vec<TorrentFile>>,
        extra_paths: Mutex<Vec<String>>,
        fail_directory: AtomicBool,
        present: AtomicBool,
        renamed: AtomicUsize,
        rechecked: AtomicUsize,
        lost_reply: AtomicBool,
    }
    #[async_trait]
    impl DownloadClient for Client {
        async fn directory_entries(
            &self,
            _: &AppState,
            directory: &str,
        ) -> Result<Vec<String>, Error> {
            if self.fail_directory.load(Ordering::SeqCst) {
                return Err((
                    StatusCode::NOT_IMPLEMENTED,
                    "Synthetic unsupported destination check".into(),
                ));
            }
            let mut listed = self
                .files
                .lock()
                .await
                .iter()
                .map(|f| format!("{directory}/{}", f.name))
                .collect::<Vec<_>>();
            listed.extend(
                self.extra_paths
                    .lock()
                    .await
                    .iter()
                    .map(|p| format!("{directory}/{p}")),
            );
            Ok(listed)
        }
        async fn files(&self, _: &AppState, _: &str) -> Result<Vec<TorrentFile>, Error> {
            Ok(self.files.lock().await.clone())
        }
        async fn rename_file(
            &self,
            _: &AppState,
            _: &str,
            old: &str,
            new: &str,
        ) -> Result<(), Error> {
            let mut files = self.files.lock().await;
            files.iter_mut().find(|f| f.name == old).unwrap().name = new.into();
            self.renamed.fetch_add(1, Ordering::SeqCst);
            if self.lost_reply.swap(false, Ordering::SeqCst) {
                return Err(conflict("Synthetic lost reply"));
            }
            Ok(())
        }
        async fn recheck(&self, _: &AppState, _: &str) -> Result<(), Error> {
            self.rechecked.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        async fn export_torrent_file(&self, _: &AppState, _: &str) -> Result<Vec<u8>, Error> {
            Ok(META.into())
        }
        async fn list_torrents(&self, _: &AppState) -> Result<Vec<QBittorrentTorrent>, Error> {
            Ok(if self.present.load(Ordering::SeqCst) {
                vec![torrent()]
            } else {
                vec![]
            })
        }
        async fn add_magnet(
            &self,
            _: &AppState,
            _: &str,
            _: &str,
            _: &str,
            _: &str,
            _: Option<i64>,
        ) -> Result<String, Error> {
            panic!("No downloads allowed")
        }
        async fn add_torrent_file(
            &self,
            _: &AppState,
            bytes: Vec<u8>,
            _: &str,
            _: &str,
            _: &str,
            _: Option<i64>,
            path: Option<&str>,
            reseed: bool,
        ) -> Result<String, Error> {
            assert!(reseed, "Recovery must request stopped mode");
            assert_eq!(path, Some("/opaque-nas/films"));
            assert_eq!(bytes, META);
            self.present.store(true, Ordering::SeqCst);
            self.files.lock().await[0].name = "old.mkv".into();
            Ok(torrent().hash)
        }
        async fn delete_torrent(&self, _: &AppState, _: &str, _: bool) -> Result<(), Error> {
            panic!("No torrent deletion in naming/recovery")
        }
    }
    fn torrent() -> QBittorrentTorrent {
        serde_json::from_value(serde_json::json!({
        "hash":crate::cardigann::torrent_v1_info_hash(META).unwrap(),"name":"Fixture.Film.2020.1080p.WEB-DL",
        "content_path":"/opaque-nas/films/old.mkv","save_path":"/opaque-nas/films","progress":1.0,"state":"stoppedUP"
    })).unwrap()
    }
    async fn fixture() -> (AppState, Arc<Client>, DownloadJob) {
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();
        let client = Arc::new(Client {
            extra_paths: Mutex::new(Vec::new()),
            fail_directory: AtomicBool::new(false),
            files: Mutex::new(vec![TorrentFile {
                index: 0,
                name: "old.mkv".into(),
                size: 9,
            }]),
            present: AtomicBool::new(true),
            renamed: AtomicUsize::new(0),
            rechecked: AtomicUsize::new(0),
            lost_reply: AtomicBool::new(false),
        });
        let state = AppState {
            db,
            http: reqwest::Client::new(),
            automation_runtime: Default::default(),
            download_client: client.clone(),
            torrent_list_cache: Arc::new(Mutex::new(None)),
            grab_lock: Arc::new(Mutex::new(())),
        };
        sqlx::query("INSERT INTO movies(id,tmdb_id,title,year) VALUES(1,999,'Fixture Film',2020)")
            .execute(&state.db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO download_jobs(id,media_type,media_id,release_title,indexer_id,qb_hash,status) VALUES(1,'movie',1,'Fixture.Film.2020.1080p.WEB-DL','fixture',?,'completed')").bind(torrent().hash).execute(&state.db).await.unwrap();
        let job = super::super::get_job(&state, 1).await.unwrap().unwrap();
        (state, client, job)
    }
    #[tokio::test]
    async fn default_is_off_even_when_legacy_rename_was_on_and_archive_is_recoverable() {
        let (state, client, job) = fixture().await;
        settings::set_value(&state.db, "import.rename_enabled", "true")
            .await
            .unwrap();
        let old_form = serde_json::from_value(serde_json::json!({"rename_enabled":true})).unwrap();
        settings::update_settings(axum::extract::State(state.clone()), axum::Json(old_form))
            .await
            .unwrap();
        assert!(
            !settings::get_settings(axum::extract::State(state.clone()))
                .await
                .unwrap()
                .0
                .rename_enabled
        );
        settings::set_value(&state.db, "library.source", "cinetta")
            .await
            .unwrap();
        complete(&state, &job, &torrent()).await.unwrap();
        assert_eq!(client.renamed.load(Ordering::SeqCst), 0);
        assert_eq!(client.files.lock().await[0].name, "old.mkv");
        let stored: Vec<u8> = sqlx::query_scalar(
            "SELECT torrent_data FROM torrent_archives WHERE job_id=1 AND completed=1",
        )
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(stored, META);
        assert_eq!(
            restore(&state, &job).await.unwrap_err().0,
            StatusCode::CONFLICT
        );
        client.present.store(false, Ordering::SeqCst);
        restore(&state, &job).await.unwrap();
        assert_eq!(client.renamed.load(Ordering::SeqCst), 0);
        assert_eq!(client.rechecked.load(Ordering::SeqCst), 1);
    }
    #[tokio::test]
    async fn naming_and_remote_recovery_work_without_local_media_access() {
        let (state, client, job) = fixture().await;
        settings::set_value(&state.db, "qbittorrent.rename_enabled", "true")
            .await
            .unwrap();
        complete(&state, &job, &torrent()).await.unwrap();
        let named = client.files.lock().await[0].name.clone();
        assert!(named.starts_with("Fixture Film (2020)"));
        assert_eq!(client.renamed.load(Ordering::SeqCst), 1);
        client.present.store(false, Ordering::SeqCst);
        restore(&state, &job).await.unwrap();
        assert_eq!(client.files.lock().await[0].name, named);
        assert_eq!(client.rechecked.load(Ordering::SeqCst), 1);
        assert_eq!(client.renamed.load(Ordering::SeqCst), 2);
    }
    #[tokio::test]
    async fn journal_survives_lost_reply_and_retries_without_renaming_twice() {
        let (state, client, job) = fixture().await;
        settings::set_value(&state.db, "qbittorrent.rename_enabled", "true")
            .await
            .unwrap();
        client.lost_reply.store(true, Ordering::SeqCst);
        assert!(complete(&state, &job, &torrent()).await.is_err());
        let pending: i64 =
            sqlx::query_scalar("SELECT completed FROM torrent_archives WHERE job_id=1")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(pending, 0);
        settings::set_value(&state.db, "qbittorrent.rename_enabled", "false")
            .await
            .unwrap();
        assert!(complete(&state, &job, &torrent()).await.is_err());
        settings::set_value(&state.db, "qbittorrent.rename_enabled", "true")
            .await
            .unwrap();
        complete(&state, &job, &torrent()).await.unwrap();
        assert_eq!(client.renamed.load(Ordering::SeqCst), 1);
    }
    #[tokio::test]
    async fn physical_collision_or_unavailable_remote_listing_blocks_all_renames() {
        let (state, client, job) = fixture().await;
        settings::set_value(&state.db, "qbittorrent.rename_enabled", "true")
            .await
            .unwrap();
        let target = plan(
            &client.files.lock().await,
            &job,
            "Fixture Film",
            Some(2020),
            "{Title} ({Year}) - {Resolution} {Source} {Codec}",
            true,
        )
        .unwrap()[0]
            .target
            .clone();
        client.extra_paths.lock().await.push(target);
        assert_eq!(
            complete(&state, &job, &torrent()).await.unwrap_err().0,
            StatusCode::CONFLICT
        );
        assert_eq!(client.renamed.load(Ordering::SeqCst), 0);
        client.extra_paths.lock().await.clear();
        client.fail_directory.store(true, Ordering::SeqCst);
        assert_eq!(
            complete(&state, &job, &torrent()).await.unwrap_err().0,
            StatusCode::NOT_IMPLEMENTED
        );
        assert_eq!(client.renamed.load(Ordering::SeqCst), 0);
        assert_eq!(client.files.lock().await[0].name, "old.mkv");
    }

    #[tokio::test]
    async fn naming_rejects_ambiguity_sidecars_traversal_and_duplicate_targets_before_changes() {
        let (state, client, job) = fixture().await;
        let file = TorrentFile {
            index: 0,
            name: "old.mkv".into(),
            size: 9,
        };
        let extra = TorrentFile {
            index: 1,
            name: "other.mkv".into(),
            size: 9,
        };
        assert!(
            plan(
                &[file.clone(), extra],
                &job,
                "Film",
                Some(2020),
                "{Title}",
                true
            )
            .is_err()
        );
        let subtitle = TorrentFile {
            index: 1,
            name: "old.srt".into(),
            size: 2,
        };
        assert!(
            plan(
                &[file.clone(), subtitle],
                &job,
                "Film",
                None,
                "{Title}",
                true
            )
            .is_err()
        );
        for bad in [
            "../evil.mkv",
            "/evil.mkv",
            "C:/evil.mkv",
            "a\\evil.mkv",
            "a/../evil.mkv",
            "a\0.mkv",
        ] {
            assert!(!valid_relative(bad));
        }
        let mut series = job;
        series.media_type = "series".into();
        let files = [
            TorrentFile {
                index: 0,
                name: "Show.S01E01.mkv".into(),
                size: 9,
            },
            TorrentFile {
                index: 1,
                name: "Show.S01E02.mkv".into(),
                size: 9,
            },
        ];
        assert!(plan(&files, &series, "Show", None, "{Title}", true).is_err());
        let range = [TorrentFile {
            index: 0,
            name: "Show.S01E01-E02.mkv".into(),
            size: 9,
        }];
        let named = plan(
            &range,
            &series,
            "Show",
            None,
            "{Title} S{Season:00}E{Episode:00}",
            true,
        )
        .unwrap();
        assert_eq!(named[0].target, "Show S01E01-E02.mkv");
        assert_eq!(client.renamed.load(Ordering::SeqCst), 0);
        drop(state);
    }
}
