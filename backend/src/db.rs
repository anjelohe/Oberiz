use sqlx::{
    SqlitePool,
    migrate::{MigrateError, Migrator},
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteSynchronous},
};
use std::{
    path::PathBuf,
    str::FromStr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

// These were the migrations present during pre-release development, before
// Oberiz had a stable upgrade history. Only that closed set is eligible for
// legacy checksum repair; a mismatch in any later migration remains a hard
// failure so a changed or tampered release migration cannot be silently
// accepted.
const LAST_LEGACY_MIGRATION: i64 = 16;

fn data_directory() -> PathBuf {
    std::env::var_os("OBERIZ_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"))
}

pub async fn connect() -> anyhow::Result<SqlitePool> {
    let data_directory = data_directory();
    std::fs::create_dir_all(&data_directory)?;
    let database_path = data_directory.join("oberiz.db");
    let database_url = format!(
        "sqlite://{}",
        database_path.to_string_lossy().replace('\\', "/")
    );
    let options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        // Several schedulers, the web UI and RSS all hit the same file; without
        // this a momentary writer lock surfaces as a raw "database is locked"
        // error instead of the caller just waiting briefly for its turn.
        .busy_timeout(Duration::from_secs(10))
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(&pool)
        .await?;

    run_migrations_with_legacy_checksum_repair(&pool, &database_path).await?;

    Ok(pool)
}

async fn run_migrations_with_legacy_checksum_repair(
    pool: &SqlitePool,
    database_path: &std::path::Path,
) -> anyhow::Result<()> {
    let mut backup_created = false;
    loop {
        match MIGRATOR.run(pool).await {
            Ok(()) => return Ok(()),
            // Development databases created before v1.0.0 can contain the
            // same schema with pre-release migration checksums. SQLx refuses
            // to proceed until those historical signatures are reconciled.
            Err(MigrateError::VersionMismatch(version)) => {
                repair_legacy_migration_checksum(pool, database_path, version, !backup_created)
                    .await?;
                backup_created = true;
            }
            Err(error) => return Err(error.into()),
        }
    }
}

async fn repair_legacy_migration_checksum(
    pool: &SqlitePool,
    database_path: &std::path::Path,
    version: i64,
    create_backup: bool,
) -> anyhow::Result<()> {
    let base_table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('settings', 'movies')",
    )
    .fetch_one(pool)
    .await?;
    if base_table_count != 2 {
        anyhow::bail!(
            "legacy migration checksum mismatch found on an unrecognised database schema"
        );
    }

    let highest_applied_version: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations")
            .fetch_one(pool)
            .await?;
    if !(1..=LAST_LEGACY_MIGRATION).contains(&version)
        || highest_applied_version != LAST_LEGACY_MIGRATION
    {
        anyhow::bail!(
            "migration checksum mismatch for version {version} is not a recognised pre-release database"
        );
    }

    if create_backup {
        // Fold the WAL into the database before taking the safety copy so it
        // is a standalone SQLite file rather than a main file missing recent
        // changes.
        sqlx::query("PRAGMA wal_checkpoint(FULL)")
            .execute(pool)
            .await?;
        let backup_directory = database_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("backups");
        std::fs::create_dir_all(&backup_directory)?;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        std::fs::copy(
            database_path,
            backup_directory.join(format!("oberiz-before-migration-repair-{timestamp}.db")),
        )?;
    }

    let expected_checksum = MIGRATOR
        .iter()
        .find(|migration| migration.version == version)
        .ok_or_else(|| anyhow::anyhow!("unknown migration version {version} in legacy database"))?
        .checksum
        .as_ref();
    sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
        .bind(expected_checksum)
        .bind(version)
        .execute(pool)
        .await?;
    Ok(())
}
