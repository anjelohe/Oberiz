use super::*;
use std::sync::atomic::Ordering;

static FETCH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

impl Sources {
    pub(super) async fn metadata(&self, path: &str) -> Result<Value> {
        // Serialize the bounded report workers, including cache misses, to avoid
        // duplicate provider requests and unbounded background refresh tasks.
        let _guard = FETCH_LOCK.lock().await;
        let key = format!("{}|es-ES|{path}", self.tmdb_base);
        let now: i64 = sqlx::query_scalar("SELECT unixepoch()")
            .fetch_one(&self.db)
            .await
            .map_err(db_error)?;
        let cached: Option<(String, i64, bool)> = sqlx::query_as(
            "SELECT data_json,fetched_at,final_series FROM missing_metadata_cache WHERE cache_key=?")
            .bind(&key).fetch_optional(&self.db).await.map_err(db_error)?;
        let final_series = if path.starts_with("/tv/") && path.contains("/season/") {
            let parent = path.split("/season/").next().unwrap_or(path);
            sqlx::query_scalar::<_, bool>(
                "SELECT final_series FROM missing_metadata_cache WHERE cache_key=?",
            )
            .bind(format!("{}|es-ES|{parent}", self.tmdb_base))
            .fetch_optional(&self.db)
            .await
            .map_err(db_error)?
            .unwrap_or(false)
        } else {
            cached.as_ref().is_some_and(|c| c.2)
        };
        let days = if final_series {
            self.completed_days
        } else {
            self.metadata_days
        };
        if let Some((encoded, fetched, _)) = &cached
            && now.saturating_sub(*fetched) < (days * 86400) as i64
            && let Ok(value) = serde_json::from_str(encoded)
        {
            sqlx::query("UPDATE missing_metadata_cache SET accessed_at=? WHERE cache_key=? AND accessed_at<?")
                .bind(now)
                .bind(&key)
                .bind(now-3600)
                .execute(&self.db)
                .await
                .map_err(db_error)?;
            return Ok(value);
        }
        let value = match self.fetch_metadata(path).await {
            Ok(value) => {
                let valid = if path.starts_with("/movie/") {
                    value.get("belongs_to_collection").is_some()
                } else if path.starts_with("/collection/") {
                    value["parts"].is_array()
                } else if path.contains("/season/") {
                    value["episodes"].is_array()
                } else if path.starts_with("/tv/") {
                    value["seasons"].is_array()
                } else {
                    value["tv_results"].is_array()
                };
                if !valid {
                    return Err(err(StatusCode::BAD_GATEWAY, "invalid_provider_response"));
                }
                compact(path, value)
            }
            Err(error) => {
                if let Some((encoded, fetched, _)) = cached
                    && now.saturating_sub(fetched) <= 365 * 86400
                    && let Ok(value) = serde_json::from_str(&encoded)
                {
                    self.stale_metadata.store(true, Ordering::Relaxed);
                    return Ok(value);
                }
                return Err(error);
            }
        };
        let encoded = value.to_string();
        let ended = matches!(value["status"].as_str(), Some("Ended" | "Canceled"));
        let mut tx = self.db.begin().await.map_err(db_error)?;
        sqlx::query("INSERT INTO missing_metadata_cache(cache_key,data_json,fetched_at,accessed_at,final_series) VALUES(?,?,?,?,?) ON CONFLICT(cache_key) DO UPDATE SET data_json=excluded.data_json,fetched_at=excluded.fetched_at,accessed_at=excluded.accessed_at,final_series=excluded.final_series")
            .bind(&key).bind(encoded).bind(now).bind(now).bind(ended)
            .execute(&mut *tx).await.map_err(db_error)?;
        loop {
            let (count, bytes): (i64, i64) = sqlx::query_as(
                "SELECT count(*),COALESCE(sum(length(data_json)),0) FROM missing_metadata_cache",
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(db_error)?;
            if count <= 20_000 && bytes <= 128 * 1024 * 1024 {
                break;
            }
            sqlx::query("DELETE FROM missing_metadata_cache WHERE cache_key IN (SELECT cache_key FROM missing_metadata_cache ORDER BY accessed_at,cache_key LIMIT 100)")
                .execute(&mut *tx).await.map_err(db_error)?;
        }
        tx.commit().await.map_err(db_error)?;
        Ok(value)
    }
}

// Reports do not need posters, credits, biographies, or full provider responses.
fn compact(path: &str, value: Value) -> Value {
    if path.starts_with("/movie/") {
        json!({"belongs_to_collection": value["belongs_to_collection"]})
    } else if path.starts_with("/collection/") {
        json!({"name":value["name"],"parts":value["parts"].as_array().map(|parts| parts.iter().map(|p| json!({"id":p["id"],"title":p["title"],"release_date":p["release_date"]})).collect::<Vec<_>>())})
    } else if path.contains("/season/") {
        json!({"episodes":value["episodes"].as_array().map(|rows| rows.iter().map(|e| json!({"id":e["id"],"name":e["name"],"episode_number":e["episode_number"],"air_date":e["air_date"]})).collect::<Vec<_>>())})
    } else if path.starts_with("/tv/") {
        json!({"name":value["name"],"status":value["status"],"seasons":value["seasons"].as_array().map(|rows| rows.iter().map(|s| json!({"season_number":s["season_number"]})).collect::<Vec<_>>())})
    } else {
        value
    }
}
