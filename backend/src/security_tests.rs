//! Regression fixtures never contact real integrations or application data.
use crate::*;

#[tokio::test]
async fn backup_upload_stream_validates_header_and_size_without_memory_buffer() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    let directory = std::env::temp_dir().join(format!(
        "oberiz-upload-fixture-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let invalid = Request::builder()
        .body(Body::from("not a sqlite database"))
        .unwrap();
    assert_eq!(
        backups::receive_backup(invalid, directory.clone())
            .await
            .unwrap_err()
            .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
    let huge = Request::builder()
        .header("content-length", "536870913")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        backups::receive_backup(huge, directory.clone())
            .await
            .unwrap_err()
            .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
    let valid = Request::builder()
        .body(Body::from(&b"SQLite format 3\0synthetic fixture"[..]))
        .unwrap();
    let file = backups::receive_backup(valid, directory.clone())
        .await
        .unwrap();
    assert_eq!(
        std::fs::read(&file).unwrap(),
        b"SQLite format 3\0synthetic fixture"
    );
    std::fs::remove_file(file).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[tokio::test]
async fn backup_restore_policy_preserves_current_security() {
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&db).await.unwrap();
    // Exercise the same policy helpers used inside the restore transaction,
    // without changing process-wide paths or touching the running installation.
    settings::set_value(&db, "auth.password_hash", "current-password-hash")
        .await
        .unwrap();
    settings::set_value(&db, "api.key_hash", "current-api-key-hash")
        .await
        .unwrap();
    settings::set_value(&db, "cinetta.url", "https://current.invalid")
        .await
        .unwrap();
    settings::set_value(&db, "cinetta.inventory_key", "current-inventory-key")
        .await
        .unwrap();
    // The restore implementation uses these same helpers within its transaction.
    let before = backups::security_snapshot(&mut db.acquire().await.unwrap())
        .await
        .unwrap();
    settings::set_value(&db, "auth.password_hash", "old-password-hash")
        .await
        .unwrap();
    settings::set_value(&db, "api.key_hash", "old-api-key-hash")
        .await
        .unwrap();
    settings::set_value(&db, "cinetta.url", "https://old.invalid")
        .await
        .unwrap();
    settings::set_value(&db, "qbittorrent.password", "old-qb-secret")
        .await
        .unwrap();
    backups::preserve_security(&mut db.acquire().await.unwrap(), before)
        .await
        .unwrap();
    assert_eq!(
        settings::get_value(&db, "auth.password_hash")
            .await
            .unwrap()
            .as_deref(),
        Some("current-password-hash")
    );
    assert_eq!(
        settings::get_value(&db, "api.key_hash")
            .await
            .unwrap()
            .as_deref(),
        Some("current-api-key-hash")
    );
    assert_eq!(
        settings::get_value(&db, "cinetta.url")
            .await
            .unwrap()
            .as_deref(),
        Some("https://current.invalid")
    );
    assert!(
        settings::get_value(&db, "qbittorrent.password")
            .await
            .unwrap()
            .is_none()
    );
}
