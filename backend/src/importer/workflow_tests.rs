//! Full domain workflow against a local RSS fixture and a fake download client.
//! No user database, private indexer or real torrent client is used.
use super::*;
use crate::{download_client::DownloadClient, qbittorrent::QBittorrentTorrent};
use async_trait::async_trait;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::{fs, path::PathBuf};
use tokio::sync::Mutex;

struct Folder(PathBuf);
impl Drop for Folder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct FakeClient {
    torrents: Mutex<Vec<QBittorrentTorrent>>,
    fail_add: AtomicBool,
    fail_list: AtomicBool,
    adds: AtomicUsize,
    downloads: PathBuf,
}

#[tokio::test]
async fn diagnostic_series_automation_finds_season_pack_after_complete_series_miss() {
    use axum::{extract::Query, response::Html};
    use std::collections::HashMap;
    let folder = Folder(std::env::temp_dir().join(format!(
            "oberiz-series-diagnostic-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    fs::create_dir_all(&folder.0).unwrap();
    let client = Arc::new(FakeClient {
        torrents: Mutex::new(vec![]),
        fail_add: AtomicBool::new(false),
        fail_list: AtomicBool::new(false),
        adds: AtomicUsize::new(0),
        downloads: folder.0.clone(),
    });
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    let state = AppState {
        db,
        http: reqwest::Client::builder().no_proxy().build().unwrap(),
        automation_runtime: Default::default(),
        download_client: client.clone(),
        torrent_list_cache: Arc::new(Mutex::new(Some(crate::TorrentListCache {
            fetched_at: std::time::Instant::now(),
            torrents: vec![],
        }))),
        grab_lock: Arc::new(Mutex::new(())),
    };
    settings::set_value(&state.db, "library.source", "cinetta")
        .await
        .unwrap();
    settings::set_value(&state.db, "import.keep_reseed_metadata", "false")
        .await
        .unwrap();
    settings::set_value(
        &state.db,
        "paths.custom_indexers",
        &folder.0.to_string_lossy(),
    )
    .await
    .unwrap();
    settings::set_value(
        &state.db,
        "paths.upstream_indexers",
        &folder.0.to_string_lossy(),
    )
    .await
    .unwrap();
    sqlx::query("UPDATE quality_profiles SET language_profile_id=NULL,min_seeders=1,max_size_mb=5000,max_season_pack_size_mb=50000,upgrade_allowed=0,rules_json=? WHERE id=4")
        .bind(r#"{"resolutions":{"1080P":10},"series_prefer_pack":true,"series_accept_complete":true}"#).execute(&state.db).await.unwrap();
    sqlx::query("INSERT INTO series(id,tmdb_id,name,original_name,year,quality_profile_id,monitored,monitor_mode) VALUES(1,203667,'Reina roja','Reina roja',2024,4,1,'future')").execute(&state.db).await.unwrap();
    sqlx::query("INSERT INTO series_seasons(id,series_id,season_number,name,episode_count,monitored) VALUES(1,1,2,'Loba Negra',6,1)").execute(&state.db).await.unwrap();
    for ep in 1..=6 {
        sqlx::query("INSERT INTO series_episodes(series_id,season_id,season_number,episode_number,name,air_date,monitored) VALUES(1,1,2,?,'Episode',date('now','-10 days'),1)").bind(ep).execute(&state.db).await.unwrap();
    }
    let calls = Arc::new(Mutex::new(Vec::new()));
    let app = axum::Router::new().route("/search",axum::routing::get({let calls=calls.clone(); move |Query(q): Query<HashMap<String,String>>| {let calls=calls.clone(); async move {
        calls.lock().await.push(q.get("q").cloned().unwrap_or_default());
        Html("<table><tr class='release'><td><a class='title' href='/details/1'>Reina roja S02 1080p AMZN WEB-DL DD+ 5.1 H265 - TSeD</a><a class='download' href='magnet:?xt=urn:btih:3333333333333333333333333333333333333333'>Get</a></td></tr></table>")
    }}}));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    fs::write(
        folder.0.join("series-diagnostic.yml"),
        format!(
            r#"
id: series-diagnostic
name: Series diagnostic fixture
links:
  - http://{address}/
search:
  path: search
  inputs:
    q: "{{{{ .Keywords }}}}"
  rows:
    selector: tr.release
  fields:
    title:
      selector: a.title
    details:
      selector: a.title
      attribute: href
    download:
      selector: a.download
      attribute: href
    size:
      text: 9 GB
    seeders:
      text: 18
"#
        ),
    )
    .unwrap();
    sqlx::query("INSERT INTO indexer_configs(indexer_id,enabled,config_json) VALUES('series-diagnostic',1,'{}')").execute(&state.db).await.unwrap();
    let summary = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        crate::automation::run_cycle(&state),
    )
    .await
    .expect("automation cycle did not finish");
    let errors: Vec<(String, Option<String>)> =
        sqlx::query_as("SELECT status,last_error FROM series_target_state")
            .fetch_all(&state.db)
            .await
            .unwrap();
    assert_eq!(
        summary.grabbed,
        1,
        "{summary:?}; {errors:?}; {:?}",
        calls.lock().await
    );
    assert_eq!(client.adds.load(Ordering::SeqCst), 1);
    assert!(calls.lock().await.iter().any(|q| q == "Reina roja S02"));
    server.abort();
}
#[async_trait]
impl DownloadClient for FakeClient {
    async fn list_torrents(
        &self,
        _: &AppState,
    ) -> Result<Vec<QBittorrentTorrent>, (StatusCode, String)> {
        if self.fail_list.load(Ordering::SeqCst) {
            return Err((StatusCode::BAD_GATEWAY, "Fixture client unavailable".into()));
        }
        Ok(self.torrents.lock().await.clone())
    }
    async fn add_magnet(
        &self,
        state: &AppState,
        url: &str,
        category: &str,
        title: &str,
        tags: &str,
        _: Option<i64>,
    ) -> Result<String, (StatusCode, String)> {
        if self.fail_add.swap(false, Ordering::SeqCst) {
            return Err((StatusCode::BAD_GATEWAY, "Fixture add failure".into()));
        }
        self.adds.fetch_add(1, Ordering::SeqCst);
        let hash = qbittorrent::magnet_hex_hash(url).unwrap();
        let file = self.downloads.join(format!("{title}.mkv"));
        fs::write(&file, b"controlled media fixture").unwrap();
        let torrent=serde_json::from_value(serde_json::json!({"hash":hash,"name":title,"category":category,"tags":tags,"content_path":file,"save_path":self.downloads,"progress":1.0,"state":"pausedUP"})).unwrap();
        self.torrents.lock().await.push(torrent);
        *state.torrent_list_cache.lock().await = Some(crate::TorrentListCache {
            fetched_at: std::time::Instant::now(),
            torrents: self.torrents.lock().await.clone(),
        });
        Ok(hash)
    }
    async fn add_torrent_file(
        &self,
        _: &AppState,
        _: Vec<u8>,
        _: &str,
        _: &str,
        _: &str,
        _: Option<i64>,
        _: Option<&str>,
        _: bool,
    ) -> Result<String, (StatusCode, String)> {
        panic!("Unexpected torrent-file add");
    }
    async fn export_torrent_file(
        &self,
        _: &AppState,
        _: &str,
    ) -> Result<Vec<u8>, (StatusCode, String)> {
        panic!("Metadata export disabled in fixture");
    }
    async fn delete_torrent(
        &self,
        _: &AppState,
        _: &str,
        _: bool,
    ) -> Result<(), (StatusCode, String)> {
        panic!("Workflow must never delete existing torrents");
    }
}

#[tokio::test]
async fn rss_download_import_rescan_upgrade_and_failure_recovery() {
    let folder = Folder(std::env::temp_dir().join(format!(
            "oberiz-workflow-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    let library = folder.0.join("downloads");
    let downloads = folder.0.join("downloads");
    fs::create_dir_all(&library).unwrap();
    fs::create_dir_all(&downloads).unwrap();
    let client = Arc::new(FakeClient {
        torrents: Mutex::new(vec![]),
        fail_add: AtomicBool::new(true),
        fail_list: AtomicBool::new(false),
        adds: AtomicUsize::new(0),
        downloads,
    });
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    let state = AppState {
        db,
        http: reqwest::Client::new(),
        automation_runtime: Default::default(),
        download_client: client.clone(),
        torrent_list_cache: Arc::new(Mutex::new(Some(crate::TorrentListCache {
            fetched_at: std::time::Instant::now(),
            torrents: vec![],
        }))),
        grab_lock: Arc::new(Mutex::new(())),
    };
    for (key, value) in [
        ("paths.movies", library.to_string_lossy().into_owned()),
        (
            "library.movie_roots",
            serde_json::to_string(&vec![library.to_string_lossy()]).unwrap(),
        ),
        ("import.method", "copy".into()),
        ("import.keep_reseed_metadata", "false".into()),
    ] {
        settings::set_value(&state.db, key, &value).await.unwrap();
    }
    let profile: i64 =
        sqlx::query_scalar("SELECT id FROM quality_profiles WHERE media_type='movie' LIMIT 1")
            .fetch_one(&state.db)
            .await
            .unwrap();
    sqlx::query("UPDATE quality_profiles SET language_profile_id=NULL,rules_json=?,upgrade_allowed=1,cutoff_score=9999,qbittorrent_category='fixture' WHERE id=?").bind(r#"{"resolutions":{"1080P":10,"2160P":100}}"#).bind(profile).execute(&state.db).await.unwrap();
    sqlx::query("INSERT INTO movies(id,tmdb_id,title,year,quality_profile_id,monitored) VALUES(1,321,'Fixture Film',2020,?,1)").bind(profile).execute(&state.db).await.unwrap();

    let phase = Arc::new(AtomicUsize::new(0));
    let failure = Arc::new(AtomicBool::new(true));
    let app=axum::Router::new().route("/rss",axum::routing::get({let phase=phase.clone();let failure=failure.clone();move || {let phase=phase.clone();let failure=failure.clone();async move {
        if failure.load(Ordering::SeqCst) {return (StatusCode::SERVICE_UNAVAILABLE,"Fixture feed unavailable".to_owned());}
        let p=phase.load(Ordering::SeqCst);let resolution=if p==0 {"1080p"} else {"2160p"};let hash=if p==0 {"1111111111111111111111111111111111111111"} else {"2222222222222222222222222222222222222222"};
        (StatusCode::OK,format!("<rss><channel><item><guid>fixture-{p}</guid><title>Fixture.Film.2020.{resolution}.WEB-DL</title><enclosure url=\"magnet:?xt=urn:btih:{hash}\"/></item></channel></rss>"))
    }}}));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/rss", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    sqlx::query(
        "INSERT INTO indexer_configs(indexer_id,enabled,config_json) VALUES('fixture',1,?)",
    )
    .bind(serde_json::json!({"rss_url":url}).to_string())
    .execute(&state.db)
    .await
    .unwrap();

    assert_eq!(crate::rss::run_cycle(&state).await.errors, 1); // HTTP failure is retryable.
    failure.store(false, Ordering::SeqCst);
    assert_eq!(crate::rss::run_cycle(&state).await.errors, 1); // Download-client add failure.
    assert_eq!(crate::rss::run_cycle(&state).await.grabbed, 1); // Same RSS item retries.
    assert_eq!(client.adds.load(Ordering::SeqCst), 1);
    assert_eq!(crate::rss::run_cycle(&state).await.new_items, 0); // Durable feed deduplication.
    let source = PathBuf::from(&client.torrents.lock().await[0].content_path);
    let offline = folder.0.join("offline-source");
    fs::rename(&source, &offline).unwrap();
    run_cycle(&state).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM media_files")
            .fetch_one(&state.db)
            .await
            .unwrap(),
        0
    );
    fs::rename(&offline, &source).unwrap();
    run_cycle(&state).await.unwrap();
    assert!(source.exists()); // Tracking never opens or copies the media.
    crate::library::scan_media(&state, "movie").await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM media_files WHERE file_exists=1")
            .fetch_one(&state.db)
            .await
            .unwrap(),
        1
    );
    crate::library::scan_media(&state, "movie").await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM media_files WHERE file_exists=1")
            .fetch_one(&state.db)
            .await
            .unwrap(),
        1
    );

    // A completed sharing policy releases the active-job gate, allowing a better release.
    sqlx::query("UPDATE download_jobs SET status='cleaned',cleaned_at=CURRENT_TIMESTAMP WHERE imported_at IS NOT NULL").execute(&state.db).await.unwrap();
    phase.store(1, Ordering::SeqCst);
    let upgrade = crate::rss::run_cycle(&state).await;
    let outcomes: Vec<(String, String)> = sqlx::query_as(
        "SELECT outcome,COALESCE(detail,'') FROM rss_processed_releases ORDER BY release_guid",
    )
    .fetch_all(&state.db)
    .await
    .unwrap();
    assert_eq!(upgrade.grabbed, 1, "{outcomes:?}");
    run_cycle(&state).await.unwrap();
    assert_eq!(client.adds.load(Ordering::SeqCst), 2);
    crate::library::scan_media(&state, "movie").await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM media_files WHERE file_exists=1")
            .fetch_one(&state.db)
            .await
            .unwrap(),
        2
    );
    let statuses: Vec<String> = sqlx::query_scalar("SELECT status FROM download_jobs ORDER BY id")
        .fetch_all(&state.db)
        .await
        .unwrap();
    client.fail_list.store(true, Ordering::SeqCst);
    assert!(reconcile_missing_torrents(&state).await.is_err());
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT status FROM download_jobs ORDER BY id")
            .fetch_all(&state.db)
            .await
            .unwrap(),
        statuses
    );
    client.fail_list.store(false, Ordering::SeqCst);
    let disconnected = folder.0.join("disconnected-library");
    fs::rename(&library, &disconnected).unwrap();
    assert!(
        crate::library::scan_media(&state, "movie").await.unwrap()[0]
            .error
            .is_some()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM media_files WHERE file_exists=1")
            .fetch_one(&state.db)
            .await
            .unwrap(),
        2
    );
    assert!(
        client
            .torrents
            .lock()
            .await
            .iter()
            .all(|torrent| torrent.state == "pausedUP")
    );
    server.abort();
}
