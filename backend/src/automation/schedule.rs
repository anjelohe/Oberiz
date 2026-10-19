//! Persisted random daily plans. Missed cycles never cause catch-up bursts.
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize)]
struct Plan {
    start: i64,
    end: i64,
    count: i64,
    slots: Vec<i64>,
}

fn generate(start: i64, end: i64, count: i64) -> Plan {
    let slots = (0..count)
        .map(|i| {
            let width = (end - start) / count;
            start
                + (end - start) * i / count
                + width / 4
                + (OsRng.next_u64() % (width / 2).max(1) as u64) as i64
        })
        .collect();
    Plan {
        start,
        end,
        count,
        slots,
    }
}

async fn update(
    db: &SqlitePool,
    count: i64,
    now: Option<i64>,
    manual: bool,
) -> Result<bool, sqlx::Error> {
    let count = count.clamp(1, 24);
    let mut tx = db.begin().await?;
    let now: i64 = match now {
        Some(now) => now,
        None => {
            sqlx::query_scalar("SELECT unixepoch()")
                .fetch_one(&mut *tx)
                .await?
        }
    };
    // Civil-day boundaries in the host timezone, including 23/25-hour DST days.
    let (start,end): (i64,i64)=sqlx::query_as("SELECT CAST(strftime('%s',datetime(?,'unixepoch','localtime','start of day'),'utc') AS INTEGER),CAST(strftime('%s',datetime(?,'unixepoch','localtime','start of day','+1 day'),'utc') AS INTEGER)")
        .bind(now).bind(now).fetch_one(&mut *tx).await?;
    sqlx::query("INSERT OR IGNORE INTO settings(key,value) VALUES('automation.daily_plan','')")
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE settings SET value=value WHERE key='automation.daily_plan'")
        .execute(&mut *tx)
        .await?;
    let raw: String =
        sqlx::query_scalar("SELECT value FROM settings WHERE key='automation.daily_plan'")
            .fetch_one(&mut *tx)
            .await?;
    let previous = serde_json::from_str::<Plan>(&raw).ok().filter(|p| {
        p.start == start
            && p.end == end
            && p.count == count
            && p.slots.iter().all(|s| *s >= start && *s < end)
    });
    let fresh = previous.is_none();
    let mut plan = previous.unwrap_or_else(|| generate(start, end, count));
    let due = !manual && !fresh && plan.slots.first().is_some_and(|s| *s <= now);
    // Persist consumption BEFORE network work. At most one overdue cycle runs.
    plan.slots.retain(|s| *s > now);
    let encoded = serde_json::to_string(&plan).expect("serializable plan");
    sqlx::query("UPDATE settings SET value=?,updated_at=CURRENT_TIMESTAMP WHERE key='automation.daily_plan'").bind(encoded).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(due)
}

pub(super) async fn claim_due(
    db: &SqlitePool,
    count: i64,
    now: Option<i64>,
) -> Result<bool, sqlx::Error> {
    update(db, count, now, false).await
}
pub(super) async fn reserve(
    db: &SqlitePool,
    count: i64,
    now: Option<i64>,
) -> Result<(), sqlx::Error> {
    update(db, count, now, true).await.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn plan(db: &SqlitePool) -> String {
        sqlx::query_scalar("SELECT value FROM settings WHERE key='automation.daily_plan'")
            .fetch_one(db)
            .await
            .unwrap()
    }
    #[test]
    fn counts_windows_and_spacing() {
        for count in [1, 2, 4, 24] {
            let p = generate(0, 86400, count);
            assert_eq!(p.slots.len(), count as usize);
            assert!(p.slots.windows(2).all(|s| s[1] - s[0] >= 86400 / count / 2));
            assert!(p.slots.iter().all(|s| *s > 0 && *s < 86400));
        }
    }
    #[tokio::test]
    async fn database_reopen_preserves_plan_and_claims_once() {
        let path = std::env::temp_dir().join(format!(
            "oberiz-daily-{}-{}.sqlite",
            std::process::id(),
            OsRng.next_u64()
        ));
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true);
        let db = SqlitePool::connect_with(options.clone()).await.unwrap();
        sqlx::query(
            "CREATE TABLE settings(key TEXT PRIMARY KEY,value TEXT NOT NULL,updated_at TEXT)",
        )
        .execute(&db)
        .await
        .unwrap();
        let now: i64 = sqlx::query_scalar("SELECT CAST(strftime('%s',datetime(1700000000,'unixepoch','localtime','start of day'),'utc') AS INTEGER)").fetch_one(&db).await.unwrap();
        assert!(!claim_due(&db, 2, Some(now)).await.unwrap());
        let original = plan(&db).await;
        db.close().await;
        let db = SqlitePool::connect_with(options).await.unwrap();
        assert!(!claim_due(&db, 2, Some(now)).await.unwrap());
        assert_eq!(plan(&db).await, original);
        let p: Plan = serde_json::from_str(&original).unwrap();
        assert_eq!(p.slots.len(), 2);
        for slot in p.slots {
            assert!(claim_due(&db, 2, Some(slot)).await.unwrap());
            assert!(!claim_due(&db, 2, Some(slot)).await.unwrap());
        }
        assert!(!claim_due(&db, 2, Some(p.end + 86400 * 4)).await.unwrap());
        reserve(&db, 2, Some(p.end + 86400 * 4)).await.unwrap();
        assert!(!claim_due(&db, 2, Some(p.end + 86400 * 4)).await.unwrap());
        assert!(!claim_due(&db, 4, Some(p.end + 86400 * 4)).await.unwrap());
        // Checkbox off: clearing a saved plan regenerates times without an
        // immediate network cycle. The default preserves it across restarts.
        sqlx::query("UPDATE settings SET value='' WHERE key='automation.daily_plan'")
            .execute(&db)
            .await
            .unwrap();
        assert!(!claim_due(&db, 4, Some(p.end + 86400 * 4)).await.unwrap());
        db.close().await;
        std::fs::remove_file(path).unwrap();
    }
}
