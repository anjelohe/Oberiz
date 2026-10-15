//! Guards against the classic self-hosted-app failure: a schema change that
//! works on a brand-new database but breaks an existing installation's data
//! on upgrade.
//!
//! `tests/fixtures/migrations_v1.0.2` is a frozen legacy-development schema
//! (commit 3ba6793), predating the repository's version reset. Its directory
//! name does not describe the new public v1.0.2 release. This test builds a database with only
//! those applied — simulating a real user's existing install — seeds it with
//! sample data, then runs the *current* migrations on top and checks both
//! that the upgrade succeeds and that the pre-existing data survives
//! untouched.
//!
//! When a future release adds another migration, freeze the migrations
//! directory as it stood at the previous release into a new
//! `migrations_vX.Y.Z` fixture the same way, so upgrades keep being tested
//! from the last real shape users actually have on disk.
use sqlx::{
    Row,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{path::PathBuf, str::FromStr};

#[tokio::test]
async fn storage_transition_preserves_history_and_never_inherits_rename_consent() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    let migrator = Migrator::new(PathBuf::from("migrations")).await.unwrap();
    for migration in migrator.iter().filter(|m| m.version < 22) {
        sqlx::raw_sql(&migration.sql).execute(&pool).await.unwrap();
    }
    for (key, value) in [
        ("import.method", "external"),
        ("import.rename_enabled", "true"),
        ("storage.smb.connections", "synthetic-retired-secret"),
    ] {
        sqlx::query("INSERT INTO settings(key,value) VALUES(?,?) ON CONFLICT(key) DO UPDATE SET value=excluded.value").bind(key).bind(value).execute(&pool).await.unwrap();
    }
    sqlx::query("INSERT INTO download_jobs(media_type,media_id,release_title,indexer_id,status,file_mappings_json) VALUES('movie',1,'Historical torrent','fixture','cleaned','[{\"original_rel\":\"original.mkv\",\"library_rel\":\"named.mkv\"}]')").execute(&pool).await.unwrap();
    sqlx::raw_sql(&migrator.iter().find(|m| m.version == 22).unwrap().sql)
        .execute(&pool)
        .await
        .unwrap();
    let value = |key: &'static str| {
        sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key=?")
            .bind(key)
            .fetch_one(&pool)
    };
    assert_eq!(value("library.source").await.unwrap(), "cinetta");
    assert_eq!(value("qbittorrent.rename_enabled").await.unwrap(), "false");
    let legacy: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM settings WHERE key='storage.smb.connections'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(legacy, 0);
    let mappings: String = sqlx::query_scalar(
        "SELECT file_mappings_json FROM download_jobs WHERE release_title='Historical torrent'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(mappings.contains("original.mkv") && mappings.contains("named.mkv"));
    sqlx::query("INSERT INTO torrent_archives(job_id,hash,save_path,torrent_data,mappings_json) VALUES(1,'synthetic','/opaque',X'0102','[]')").execute(&pool).await.unwrap();
}

async fn temp_pool() -> (sqlx::SqlitePool, PathBuf) {
    let path = std::env::temp_dir().join(format!(
        "oberiz-migration-test-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let url = format!("sqlite://{}", path.display().to_string().replace('\\', "/"));
    let options = SqliteConnectOptions::from_str(&url)
        .unwrap()
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .expect("open temp sqlite database");
    (pool, path)
}

#[tokio::test]
async fn upgrading_from_the_legacy_fixture_preserves_existing_data() {
    let (pool, path) = temp_pool().await;

    // Step 1: reconstruct the historical legacy fixture, not public v1.0.2.
    let old_migrator = Migrator::new(PathBuf::from("tests/fixtures/migrations_v1.0.2"))
        .await
        .expect("load frozen v1.0.2 migrations");
    old_migrator
        .run(&pool)
        .await
        .expect("apply v1.0.2 migrations to a fresh database");

    // Step 2: seed it like a real, already-in-use installation.
    sqlx::query("INSERT INTO settings(key, value) VALUES ('paths.movies', '/media/movies')")
        .execute(&pool)
        .await
        .expect("seed a setting");
    sqlx::query("INSERT INTO movies(tmdb_id, title, monitored) VALUES (603, 'The Matrix', 1)")
        .execute(&pool)
        .await
        .expect("seed a movie");

    // Step 3: upgrade in place, applying every migration added since.
    let current_migrator = Migrator::new(PathBuf::from("migrations"))
        .await
        .expect("load current migrations");
    current_migrator
        .run(&pool)
        .await
        .expect("upgrade an existing v1.0.2 database to the current schema");

    // Step 4: the pre-existing data must survive the upgrade untouched.
    let movies_path: String =
        sqlx::query_scalar("SELECT value FROM settings WHERE key = 'paths.movies'")
            .fetch_one(&pool)
            .await
            .expect("setting survives the upgrade");
    assert_eq!(movies_path, "/media/movies");

    let movie = sqlx::query("SELECT title, monitored FROM movies WHERE tmdb_id = 603")
        .fetch_one(&pool)
        .await
        .expect("movie survives the upgrade");
    assert_eq!(movie.get::<String, _>("title"), "The Matrix");
    assert_eq!(movie.get::<i64, _>("monitored"), 1);

    // And the newer schema must actually be present.
    let auth_sessions_table: Option<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='auth_sessions'",
    )
    .fetch_optional(&pool)
    .await
    .expect("query sqlite_master");
    assert_eq!(
        auth_sessions_table.as_deref(),
        Some("auth_sessions"),
        "migrations added after v1.0.2 must have applied on top of the existing database"
    );

    // Re-running the current migrator again must be a safe no-op (this is
    // exactly what happens on every normal Oberiz restart).
    current_migrator
        .run(&pool)
        .await
        .expect("re-running migrations on an up-to-date database must be a no-op");

    drop(pool);
    let _ = std::fs::remove_file(&path);
}
