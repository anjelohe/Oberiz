use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{AppState, cardigann::ReleaseResult, releases};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityRules {
    #[serde(default)]
    pub resolutions: HashMap<String, i32>,
    #[serde(default)]
    pub sources: HashMap<String, i32>,
    #[serde(default)]
    pub codecs: HashMap<String, i32>,
    #[serde(default)]
    pub hdr: HashMap<String, i32>,
    #[serde(default)]
    pub audio: HashMap<String, i32>,
    #[serde(default)]
    pub reject_terms: Vec<String>,
    #[serde(default)]
    pub prefer_terms: HashMap<String, i32>,
    #[serde(default = "default_true")]
    pub allow_unknown_resolution: bool,
    #[serde(default = "default_true")]
    pub allow_unknown_source: bool,
    #[serde(default)]
    pub series_prefer_pack: bool,
    #[serde(default)]
    pub prefer_indexer_priority: bool,
    #[serde(default = "default_true")]
    pub series_accept_complete: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageProfile {
    pub id: i64,
    pub name: String,
    pub allowed_languages: Vec<String>,
    pub scores: HashMap<String, i32>,
    pub allow_unknown: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityProfile {
    pub id: i64,
    pub name: String,
    pub media_type: String,
    pub enabled: bool,
    pub upgrade_allowed: bool,
    pub cutoff_score: i32,
    pub min_seeders: i64,
    pub min_size_mb: Option<i64>,
    pub max_size_mb: Option<i64>,
    pub max_season_pack_size_mb: Option<i64>,
    pub language_profile_id: Option<i64>,
    pub language_profile_name: Option<String>,
    pub qbittorrent_category: String,
    pub qbittorrent_tags_template: String,
    pub is_default: bool,
    pub request_quality: String,
    pub rules: QualityRules,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ProfileListQuery {
    pub media_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveQualityProfile {
    pub name: String,
    pub media_type: String,
    pub enabled: Option<bool>,
    pub upgrade_allowed: Option<bool>,
    pub cutoff_score: Option<i32>,
    pub min_seeders: Option<i64>,
    pub min_size_mb: Option<i64>,
    pub max_size_mb: Option<i64>,
    pub max_season_pack_size_mb: Option<i64>,
    pub language_profile_id: Option<i64>,
    pub qbittorrent_category: Option<String>,
    pub qbittorrent_tags_template: Option<String>,
    pub request_quality: Option<String>,
    pub rules: QualityRules,
}

#[derive(Debug, Deserialize)]
pub struct SaveLanguageProfile {
    pub name: String,
    pub allowed_languages: Vec<String>,
    #[serde(default)]
    pub scores: HashMap<String, i32>,
    pub allow_unknown: Option<bool>,
}

#[derive(sqlx::FromRow)]
struct QualityProfileRow {
    id: i64,
    name: String,
    media_type: String,
    enabled: bool,
    upgrade_allowed: bool,
    cutoff_score: i32,
    min_seeders: i64,
    min_size_mb: Option<i64>,
    max_size_mb: Option<i64>,
    max_season_pack_size_mb: Option<i64>,
    language_profile_id: Option<i64>,
    language_profile_name: Option<String>,
    qbittorrent_category: String,
    qbittorrent_tags_template: String,
    is_default: bool,
    request_quality: String,
    rules_json: String,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct LanguageProfileRow {
    id: i64,
    name: String,
    allowed_languages_json: String,
    scores_json: String,
    allow_unknown: bool,
    created_at: String,
    updated_at: String,
}

fn quality_from_row(row: QualityProfileRow) -> QualityProfile {
    QualityProfile {
        id: row.id,
        name: row.name,
        media_type: row.media_type,
        enabled: row.enabled,
        upgrade_allowed: row.upgrade_allowed,
        cutoff_score: row.cutoff_score,
        min_seeders: row.min_seeders,
        min_size_mb: row.min_size_mb,
        max_size_mb: row.max_size_mb,
        max_season_pack_size_mb: row.max_season_pack_size_mb,
        language_profile_id: row.language_profile_id,
        language_profile_name: row.language_profile_name,
        qbittorrent_category: row.qbittorrent_category,
        qbittorrent_tags_template: row.qbittorrent_tags_template,
        is_default: row.is_default,
        request_quality: row.request_quality,
        rules: serde_json::from_str(&row.rules_json).unwrap_or_default(),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn language_from_row(row: LanguageProfileRow) -> LanguageProfile {
    LanguageProfile {
        id: row.id,
        name: row.name,
        allowed_languages: serde_json::from_str(&row.allowed_languages_json).unwrap_or_default(),
        scores: serde_json::from_str(&row.scores_json).unwrap_or_default(),
        allow_unknown: row.allow_unknown,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

pub async fn list_quality_profiles(
    State(state): State<AppState>,
    Query(query): Query<ProfileListQuery>,
) -> Result<Json<Vec<QualityProfile>>, (StatusCode, String)> {
    let rows = if let Some(media_type) = query.media_type {
        sqlx::query_as::<_, QualityProfileRow>(r#"
            SELECT q.id,q.name,q.media_type,q.enabled,q.upgrade_allowed,q.cutoff_score,
                   q.min_seeders,q.min_size_mb,q.max_size_mb,q.max_season_pack_size_mb,q.language_profile_id,
                   l.name AS language_profile_name,q.qbittorrent_category,q.qbittorrent_tags_template,q.is_default,q.request_quality,
                   q.rules_json,q.created_at,q.updated_at
            FROM quality_profiles q
            LEFT JOIN language_profiles l ON l.id=q.language_profile_id
            WHERE q.media_type=?
            ORDER BY q.name
        "#).bind(media_type).fetch_all(&state.db).await
    } else {
        sqlx::query_as::<_, QualityProfileRow>(r#"
            SELECT q.id,q.name,q.media_type,q.enabled,q.upgrade_allowed,q.cutoff_score,
                   q.min_seeders,q.min_size_mb,q.max_size_mb,q.max_season_pack_size_mb,q.language_profile_id,
                   l.name AS language_profile_name,q.qbittorrent_category,q.qbittorrent_tags_template,q.is_default,q.request_quality,
                   q.rules_json,q.created_at,q.updated_at
            FROM quality_profiles q
            LEFT JOIN language_profiles l ON l.id=q.language_profile_id
            ORDER BY q.media_type,q.name
        "#).fetch_all(&state.db).await
    }.map_err(internal)?;

    Ok(Json(rows.into_iter().map(quality_from_row).collect()))
}

pub(crate) async fn list_profiles_internal(
    db: &sqlx::SqlitePool,
    media_type: &str,
) -> Result<Vec<QualityProfile>, (StatusCode, String)> {
    let ids = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM quality_profiles WHERE media_type=? ORDER BY name",
    )
    .bind(media_type)
    .fetch_all(db)
    .await
    .map_err(internal)?;
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        out.push(get_quality_profile_by_id(db, id).await?);
    }
    Ok(out)
}

pub async fn create_quality_profile(
    State(state): State<AppState>,
    Json(payload): Json<SaveQualityProfile>,
) -> Result<(StatusCode, Json<QualityProfile>), (StatusCode, String)> {
    validate_media_type(&payload.media_type)?;
    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "El nombre es obligatorio".into()));
    }
    let rules_json = serde_json::to_string(&payload.rules).map_err(internal)?;
    let result = sqlx::query(r#"
        INSERT INTO quality_profiles(
          name,media_type,enabled,upgrade_allowed,cutoff_score,min_seeders,min_size_mb,max_size_mb,max_season_pack_size_mb,
          language_profile_id,qbittorrent_category,qbittorrent_tags_template,request_quality,rules_json
        )
        VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)
    "#)
        .bind(payload.name.trim()).bind(&payload.media_type)
        .bind(payload.enabled.unwrap_or(true)).bind(payload.upgrade_allowed.unwrap_or(true))
        .bind(payload.cutoff_score.unwrap_or(500)).bind(payload.min_seeders.unwrap_or(1))
        .bind(payload.min_size_mb).bind(payload.max_size_mb).bind(payload.max_season_pack_size_mb).bind(payload.language_profile_id)
        .bind(payload.qbittorrent_category.as_deref().unwrap_or("").trim())
        .bind(payload.qbittorrent_tags_template.as_deref().unwrap_or("[tracker]").trim())
        .bind(request_quality(&payload))
        .bind(rules_json).execute(&state.db).await.map_err(db_conflict)?;
    let profile = get_quality_profile_by_id(&state.db, result.last_insert_rowid()).await?;
    Ok((StatusCode::CREATED, Json(profile)))
}

pub async fn update_quality_profile(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<SaveQualityProfile>,
) -> Result<Json<QualityProfile>, (StatusCode, String)> {
    validate_media_type(&payload.media_type)?;
    let current = get_quality_profile_by_id(&state.db, id).await?;
    if current.is_default && payload.enabled == Some(false) {
        return Err((StatusCode::CONFLICT,"El perfil por defecto no se puede desactivar. Marca primero otro perfil como predeterminado.".into()));
    }
    if current.is_default && current.media_type != payload.media_type {
        return Err((StatusCode::CONFLICT,"El perfil por defecto no puede cambiar de tipo. Marca primero otro perfil como predeterminado.".into()));
    }
    let rules_json = serde_json::to_string(&payload.rules).map_err(internal)?;
    let result=sqlx::query(r#"
        UPDATE quality_profiles SET name=?,media_type=?,enabled=?,upgrade_allowed=?,cutoff_score=?,min_seeders=?,
        min_size_mb=?,max_size_mb=?,max_season_pack_size_mb=?,language_profile_id=?,qbittorrent_category=?,qbittorrent_tags_template=?,request_quality=?,
        rules_json=?,updated_at=CURRENT_TIMESTAMP WHERE id=?
    "#)
        .bind(payload.name.trim()).bind(&payload.media_type).bind(payload.enabled.unwrap_or(true))
        .bind(payload.upgrade_allowed.unwrap_or(true)).bind(payload.cutoff_score.unwrap_or(500))
        .bind(payload.min_seeders.unwrap_or(1)).bind(payload.min_size_mb).bind(payload.max_size_mb).bind(payload.max_season_pack_size_mb)
        .bind(payload.language_profile_id)
        .bind(payload.qbittorrent_category.as_deref().unwrap_or("").trim())
        .bind(payload.qbittorrent_tags_template.as_deref().unwrap_or("[tracker]").trim())
        .bind(request_quality(&payload))
        .bind(rules_json).bind(id)
        .execute(&state.db).await.map_err(db_conflict)?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Perfil no encontrado".into()));
    }
    Ok(Json(get_quality_profile_by_id(&state.db, id).await?))
}

pub async fn delete_quality_profile(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let is_default =
        sqlx::query_scalar::<_, bool>("SELECT is_default FROM quality_profiles WHERE id=?")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Perfil no encontrado".into()))?;
    if is_default {
        return Err((StatusCode::CONFLICT,"No se puede borrar el perfil por defecto. Marca primero otro perfil como predeterminado.".into()));
    }
    let used_movies: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM movies WHERE quality_profile_id=?")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .map_err(internal)?;
    let used_series: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM series WHERE quality_profile_id=?")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .map_err(internal)?;
    if used_movies + used_series > 0 {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "El perfil está asignado a {} elementos",
                used_movies + used_series
            ),
        ));
    }
    let result = sqlx::query("DELETE FROM quality_profiles WHERE id=?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Perfil no encontrado".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_language_profiles(
    State(state): State<AppState>,
) -> Result<Json<Vec<LanguageProfile>>, (StatusCode, String)> {
    Ok(Json(list_language_profiles_internal(&state.db).await?))
}

pub(crate) async fn list_language_profiles_internal(
    db: &sqlx::SqlitePool,
) -> Result<Vec<LanguageProfile>, (StatusCode, String)> {
    let rows=sqlx::query_as::<_,LanguageProfileRow>("SELECT id,name,allowed_languages_json,scores_json,allow_unknown,created_at,updated_at FROM language_profiles ORDER BY name")
        .fetch_all(db).await.map_err(internal)?;
    Ok(rows.into_iter().map(language_from_row).collect())
}

pub async fn create_language_profile(
    State(state): State<AppState>,
    Json(payload): Json<SaveLanguageProfile>,
) -> Result<(StatusCode, Json<LanguageProfile>), (StatusCode, String)> {
    if payload.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "El nombre es obligatorio".into()));
    }
    let allowed = serde_json::to_string(&payload.allowed_languages).map_err(internal)?;
    let scores = serde_json::to_string(&payload.scores).map_err(internal)?;
    let result=sqlx::query("INSERT INTO language_profiles(name,allowed_languages_json,scores_json,allow_unknown) VALUES(?,?,?,?)")
        .bind(payload.name.trim()).bind(allowed).bind(scores).bind(payload.allow_unknown.unwrap_or(true))
        .execute(&state.db).await.map_err(db_conflict)?;
    Ok((
        StatusCode::CREATED,
        Json(get_language_profile(&state.db, result.last_insert_rowid()).await?),
    ))
}

pub async fn update_language_profile(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<SaveLanguageProfile>,
) -> Result<Json<LanguageProfile>, (StatusCode, String)> {
    let allowed = serde_json::to_string(&payload.allowed_languages).map_err(internal)?;
    let scores = serde_json::to_string(&payload.scores).map_err(internal)?;
    let result=sqlx::query("UPDATE language_profiles SET name=?,allowed_languages_json=?,scores_json=?,allow_unknown=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
        .bind(payload.name.trim()).bind(allowed).bind(scores).bind(payload.allow_unknown.unwrap_or(true)).bind(id)
        .execute(&state.db).await.map_err(db_conflict)?;
    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "Perfil de idioma no encontrado".into(),
        ));
    }
    Ok(Json(get_language_profile(&state.db, id).await?))
}

pub async fn delete_language_profile(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let used: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM quality_profiles WHERE language_profile_id=?")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .map_err(internal)?;
    if used > 0 {
        return Err((
            StatusCode::CONFLICT,
            format!("El perfil de idioma está usado por {used} perfiles de calidad"),
        ));
    }
    let result = sqlx::query("DELETE FROM language_profiles WHERE id=?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "Perfil de idioma no encontrado".into(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_quality_profile_by_id(
    db: &sqlx::SqlitePool,
    id: i64,
) -> Result<QualityProfile, (StatusCode, String)> {
    let row=sqlx::query_as::<_,QualityProfileRow>(r#"
        SELECT q.id,q.name,q.media_type,q.enabled,q.upgrade_allowed,q.cutoff_score,q.min_seeders,q.min_size_mb,q.max_size_mb,q.max_season_pack_size_mb,
               q.language_profile_id,l.name AS language_profile_name,q.qbittorrent_category,q.qbittorrent_tags_template,q.is_default,q.request_quality,
               q.rules_json,q.created_at,q.updated_at
        FROM quality_profiles q LEFT JOIN language_profiles l ON l.id=q.language_profile_id WHERE q.id=?
    "#).bind(id).fetch_optional(db).await.map_err(internal)?
      .ok_or_else(||(StatusCode::NOT_FOUND,"Perfil no encontrado".into()))?;
    Ok(quality_from_row(row))
}

fn request_quality(payload: &SaveQualityProfile) -> &'static str {
    if payload.request_quality.as_deref() == Some("4k") {
        "4k"
    } else {
        "standard"
    }
}

pub async fn set_default_quality_profile(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<QualityProfile>, (StatusCode, String)> {
    let profile = get_quality_profile_by_id(&state.db, id).await?;
    let mut tx = state.db.begin().await.map_err(internal)?;
    sqlx::query("UPDATE quality_profiles SET is_default=0 WHERE media_type=?")
        .bind(&profile.media_type)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    sqlx::query("UPDATE quality_profiles SET is_default=1,enabled=1,updated_at=CURRENT_TIMESTAMP WHERE id=?")
        .bind(id).execute(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(Json(get_quality_profile_by_id(&state.db, id).await?))
}

pub async fn default_profile_id(
    db: &sqlx::SqlitePool,
    media_type: &str,
) -> Result<Option<i64>, sqlx::Error> {
    if let Some(id) = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM quality_profiles WHERE media_type=? AND enabled=1 AND is_default=1 LIMIT 1",
    )
    .bind(media_type)
    .fetch_optional(db)
    .await?
    {
        return Ok(Some(id));
    }
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM quality_profiles WHERE media_type=? AND enabled=1 ORDER BY id LIMIT 1",
    )
    .bind(media_type)
    .fetch_optional(db)
    .await
}

pub async fn language_for_profile(
    db: &sqlx::SqlitePool,
    profile: &QualityProfile,
) -> Result<Option<LanguageProfile>, (StatusCode, String)> {
    match profile.language_profile_id {
        Some(id) => Ok(Some(get_language_profile(db, id).await?)),
        None => Ok(None),
    }
}

async fn get_language_profile(
    db: &sqlx::SqlitePool,
    id: i64,
) -> Result<LanguageProfile, (StatusCode, String)> {
    let row=sqlx::query_as::<_,LanguageProfileRow>("SELECT id,name,allowed_languages_json,scores_json,allow_unknown,created_at,updated_at FROM language_profiles WHERE id=?")
        .bind(id).fetch_optional(db).await.map_err(internal)?
        .ok_or_else(||(StatusCode::NOT_FOUND,"Perfil de idioma no encontrado".into()))?;
    Ok(language_from_row(row))
}

#[derive(Debug, Clone, Serialize)]
pub struct Evaluation {
    pub accepted: bool,
    pub match_score: i32,
    pub profile_score: i32,
    pub total_score: i32,
    pub reasons: Vec<String>,
    pub rejection_reasons: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_release(
    release: &ReleaseResult,
    profile: &QualityProfile,
    language: Option<&LanguageProfile>,
    wanted_title: &str,
    original_title: Option<&str>,
    year: Option<i32>,
    media_type: &str,
    target_season: Option<i32>,
) -> Evaluation {
    let parsed = releases::parse(&release.title);
    let mut reasons = Vec::new();
    let mut rejected = Vec::new();
    let title_match = title_match_score(&release.title, wanted_title, original_title);
    if title_match < 45 {
        rejected.push(format!("Título poco compatible ({title_match}%)"));
    }

    if let Some(expected) = year
        && let Some(found) = extract_year(&release.title)
    {
        if (found - expected).abs() > 1 {
            rejected.push(format!("Año {found}, esperado {expected}"));
        } else {
            reasons.push(format!("Año compatible {found}"));
        }
    }

    let upper = release.title.to_uppercase();
    if media_type == "movie" && looks_like_series(&upper) {
        rejected.push("Parece una serie/temporada".into());
    }
    if media_type == "series"
        && !profile.rules.series_accept_complete
        && (upper.contains("COMPLETE SERIES") || upper.contains("COMPLETE.SEASON"))
    {
        rejected.push("El perfil no acepta packs completos".into());
    }
    if media_type == "series"
        && let Some(season) = target_season
        && !matches_requested_season(&upper, season)
        // Season one is frequently published without an S01 marker. Keep
        // those valid releases, while still rejecting any explicit S02/S03.
        && (season != 1 || has_explicit_season_marker(&upper))
    {
        rejected.push(format!(
            "No corresponde a la temporada solicitada S{season:02}"
        ));
    }

    let seeds = release.seeders.unwrap_or(0);
    if seeds < profile.min_seeders {
        rejected.push(format!(
            "Solo {seeds} seeds; mínimo {}",
            profile.min_seeders
        ));
    }

    if let Some(bytes) = release.size_bytes {
        let mb = bytes / 1024 / 1024;
        if let Some(min) = profile.min_size_mb
            && mb < min
        {
            rejected.push(format!("Tamaño menor de {min} MB"));
        }
        let max = if media_type == "series" && is_season_pack_release(&upper) {
            profile.max_season_pack_size_mb.or(profile.max_size_mb)
        } else {
            profile.max_size_mb
        };
        if let Some(max) = max
            && mb > max
        {
            let label = if media_type == "series" && is_season_pack_release(&upper) {
                "temporada/pack"
            } else {
                "episodio"
            };
            rejected.push(format!("Tamaño de {label} mayor de {max} MB"));
        }
    }

    for term in &profile.rules.reject_terms {
        if contains_release_term(&upper, term) {
            rejected.push(format!("Contiene término bloqueado: {term}"));
        }
    }

    let mut profile_score = 0;
    score_dimension(
        "Resolución",
        parsed.resolution.as_deref(),
        &profile.rules.resolutions,
        profile.rules.allow_unknown_resolution,
        &mut profile_score,
        &mut reasons,
        &mut rejected,
    );
    score_dimension(
        "Fuente",
        parsed.source.as_deref(),
        &profile.rules.sources,
        profile.rules.allow_unknown_source,
        &mut profile_score,
        &mut reasons,
        &mut rejected,
    );
    score_optional(
        "Codec",
        parsed.codec.as_deref(),
        &profile.rules.codecs,
        &mut profile_score,
        &mut reasons,
    );
    score_optional(
        "HDR",
        parsed.hdr.as_deref(),
        &profile.rules.hdr,
        &mut profile_score,
        &mut reasons,
    );
    score_optional(
        "Audio",
        parsed.audio.as_deref(),
        &profile.rules.audio,
        &mut profile_score,
        &mut reasons,
    );

    for (term, score) in &profile.rules.prefer_terms {
        if contains_release_term(&upper, term) {
            profile_score += *score;
            reasons.push(format!("{term} +{score}"));
        }
    }
    if media_type == "series"
        && profile.rules.series_prefer_pack
        && (upper.contains("PACK")
            || upper.contains("COMPLETE")
            || upper.contains("SEASON")
            || upper.contains("TEMPORADA"))
    {
        profile_score += 35;
        reasons.push("Pack de serie +35".into());
    }

    if let Some(language_profile) = language {
        match parsed.language.as_deref() {
            Some(lang) => {
                let allowed: HashSet<String> = language_profile
                    .allowed_languages
                    .iter()
                    .map(|x| x.to_lowercase())
                    .collect();
                if !allowed.is_empty() && !allowed.contains(&lang.to_lowercase()) {
                    rejected.push(format!("Idioma {lang} no permitido"));
                }
                if let Some(score) = language_profile.scores.get(lang) {
                    profile_score += *score;
                    reasons.push(format!("Idioma {lang} +{score}"));
                }
            }
            None if !language_profile.allow_unknown => {
                rejected.push("Idioma no identificado".into())
            }
            None => {}
        }
    }

    let seed_bonus = (seeds.min(60) as i32).max(0);
    if seed_bonus > 0 {
        reasons.push(format!("Seeds +{seed_bonus}"));
    }
    let total = title_match + profile_score + seed_bonus;
    Evaluation {
        accepted: rejected.is_empty(),
        match_score: title_match,
        profile_score,
        total_score: total,
        reasons,
        rejection_reasons: rejected,
    }
}

fn score_dimension(
    label: &str,
    value: Option<&str>,
    scores: &HashMap<String, i32>,
    allow_unknown: bool,
    total: &mut i32,
    reasons: &mut Vec<String>,
    rejected: &mut Vec<String>,
) {
    if scores.is_empty() {
        return;
    }
    match value {
        Some(v) => match lookup_score(scores, v) {
            Some(score) => {
                *total += score;
                reasons.push(format!("{label} {v} +{score}"));
            }
            None => rejected.push(format!("{label} {v} no permitido")),
        },
        None if !allow_unknown => rejected.push(format!("{label} no identificada")),
        None => {}
    }
}

fn score_optional(
    label: &str,
    value: Option<&str>,
    scores: &HashMap<String, i32>,
    total: &mut i32,
    reasons: &mut Vec<String>,
) {
    if let Some(v) = value
        && let Some(score) = lookup_score(scores, v)
    {
        *total += score;
        reasons.push(format!("{label} {v} +{score}"));
    }
}

fn lookup_score(scores: &HashMap<String, i32>, value: &str) -> Option<i32> {
    scores
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(value))
        .map(|(_, v)| *v)
}

fn title_match_score(release: &str, title: &str, original: Option<&str>) -> i32 {
    let rel = normalize_title(release);
    let a = token_overlap(&rel, &normalize_title(title));
    let b = original
        .map(|x| token_overlap(&rel, &normalize_title(x)))
        .unwrap_or(0);
    a.max(b)
}

fn token_overlap(release: &str, wanted: &str) -> i32 {
    let wanted_tokens: Vec<&str> = wanted.split_whitespace().filter(|x| x.len() > 1).collect();
    if wanted_tokens.is_empty() {
        return 0;
    }
    let release_tokens: HashSet<&str> = release.split_whitespace().collect();
    let matched = wanted_tokens
        .iter()
        .filter(|x| release_tokens.contains(*x))
        .count();
    ((matched as f64 / wanted_tokens.len() as f64) * 100.0).round() as i32
}

fn normalize_title(value: &str) -> String {
    value
        .to_lowercase()
        .replace('á', "a")
        .replace('é', "e")
        .replace('í', "i")
        .replace('ó', "o")
        .replace('ú', "u")
        .replace('ñ', "n")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_year(value: &str) -> Option<i32> {
    value.split(|c: char| !c.is_ascii_digit()).find_map(|part| {
        if part.len() == 4 {
            let y = part.parse::<i32>().ok()?;
            if (1900..=2100).contains(&y) {
                Some(y)
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn looks_like_series(upper: &str) -> bool {
    let bytes = upper.as_bytes();
    for i in 0..bytes.len().saturating_sub(5) {
        if bytes[i] == b'S'
            && bytes
                .get(i + 1)
                .map(|x| x.is_ascii_digit())
                .unwrap_or(false)
            && bytes
                .get(i + 2)
                .map(|x| x.is_ascii_digit())
                .unwrap_or(false)
            && bytes.get(i + 3) == Some(&b'E')
            && bytes
                .get(i + 4)
                .map(|x| x.is_ascii_digit())
                .unwrap_or(false)
            && bytes
                .get(i + 5)
                .map(|x| x.is_ascii_digit())
                .unwrap_or(false)
        {
            return true;
        }
    }
    upper.contains("COMPLETE SERIES")
        || upper.contains("COMPLETE SEASON")
        || upper.contains("SEASON 1")
        || upper.contains("TEMPORADA")
}

/// A season/pack is identified only when it does not contain an explicit
/// episode marker.  This keeps `S03E07` and `3x07` under the episode limit,
/// while `S03`, `Temporada 3`, `PACK` and `Complete` use the pack limit.
fn is_season_pack_release(upper: &str) -> bool {
    let is_episode = regex::Regex::new(r"(?i)(?:\bS\s*\d{1,2}\s*E\s*\d{1,3}\b|\b\d{1,2}\s*X\s*\d{1,3}\b|\b(?:EPISODE|EPISODIO)\s*\d{1,3}\b)")
        .map(|regex| regex.is_match(upper))
        .unwrap_or(false);
    if is_episode {
        return false;
    }

    regex::Regex::new(r"(?i)(?:\bS\s*\d{1,2}\b|\b(?:SEASON|TEMPORADA)\s*\d{1,2}\b|\bPACK\b|\bCOMPLETE(?:\s+(?:SERIES|SEASON))?\b)")
        .map(|regex| regex.is_match(upper))
        .unwrap_or(false)
}

fn matches_requested_season(upper: &str, season: i32) -> bool {
    let pattern =
        format!(r"(?i)(?:\b[ST]\s*0?{season}(?:\b|E\d)|\b(?:SEASON|TEMPORADA)\s*0?{season}\b)");
    regex::Regex::new(&pattern)
        .map(|regex| regex.is_match(upper))
        .unwrap_or(false)
}

fn has_explicit_season_marker(upper: &str) -> bool {
    regex::Regex::new(r"(?i)(?:\b[ST]\s*\d{1,2}(?:\b|E\d)|\b(?:SEASON|TEMPORADA)\s*\d{1,2}\b)")
        .map(|regex| regex.is_match(upper))
        .unwrap_or(false)
}

/// Match terms as release tokens, not arbitrary character fragments.  A
/// blocked `CAM` must catch `CAM.1080p`, but must not reject a film simply
/// because its title begins with "Camino".
fn contains_release_term(release_upper: &str, term: &str) -> bool {
    let needle = term.trim().to_uppercase();
    if needle.is_empty() {
        return false;
    }

    let mut offset = 0;
    while let Some(found) = release_upper[offset..].find(&needle) {
        let start = offset + found;
        let end = start + needle.len();
        let before_is_word = release_upper[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric);
        let after_is_word = release_upper[end..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric);
        if !before_is_word && !after_is_word {
            return true;
        }
        offset = end;
    }
    false
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::{
        contains_release_term, has_explicit_season_marker, is_season_pack_release,
        matches_requested_season,
    };

    #[test]
    fn release_terms_require_token_boundaries() {
        assert!(contains_release_term("FILM.2024.CAM.1080P", "cam"));
        assert!(contains_release_term("FILM HDCAM 1080P", "HDCAM"));
        assert!(!contains_release_term("CAMINO HACIA LA LIBERTAD", "cam"));
        assert!(!contains_release_term("SCAMPER", "cam"));
    }

    #[test]
    fn requested_season_does_not_match_another_season() {
        assert!(matches_requested_season("LIONESS S03E01 1080P", 3));
        assert!(matches_requested_season("LIONESS TEMPORADA 3 1080P", 3));
        assert!(!matches_requested_season("LIONESS S01 1080P", 3));
        assert!(!matches_requested_season("LIONESS S02 1080P", 3));
    }

    #[test]
    fn first_season_can_omit_its_marker_but_not_match_another_one() {
        assert!(!has_explicit_season_marker("LIONESS 1080P WEB-DL"));
        assert!(has_explicit_season_marker("LIONESS S02 1080P WEB-DL"));
    }

    #[test]
    fn distinguishes_season_packs_from_individual_episodes() {
        assert!(is_season_pack_release("LIONESS S03 1080P WEB-DL"));
        assert!(is_season_pack_release("LIONESS TEMPORADA 3 PACK 1080P"));
        assert!(is_season_pack_release("LIONESS COMPLETE SEASON 1080P"));
        assert!(!is_season_pack_release("LIONESS S03E07 1080P WEB-DL"));
        assert!(!is_season_pack_release("LIONESS 3X07 1080P WEB-DL"));
    }
}

fn validate_media_type(value: &str) -> Result<(), (StatusCode, String)> {
    if value == "movie" || value == "series" {
        Ok(())
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            "media_type debe ser movie o series".into(),
        ))
    }
}
fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
fn db_conflict(e: sqlx::Error) -> (StatusCode, String) {
    match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => (
            StatusCode::CONFLICT,
            "Ya existe un perfil con ese nombre".into(),
        ),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}
