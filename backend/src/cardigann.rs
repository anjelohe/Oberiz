use regex::Regex;
use reqwest::{
    Client,
    header::{COOKIE, HeaderMap, HeaderName, HeaderValue},
};
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use serde_yaml::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};
use url::Url;

use crate::{AppState, releases, settings};

#[derive(Debug, Clone, Serialize)]
pub struct ReleaseResult {
    pub indexer_id: String,
    pub indexer_name: String,
    pub title: String,
    pub details_url: Option<String>,
    pub download_url: Option<String>,
    pub size_bytes: Option<i64>,
    pub seeders: Option<i64>,
    pub leechers: Option<i64>,
    pub category: Option<String>,
    pub published: Option<String>,
    pub score: i32,
    pub resolution: Option<String>,
    pub source: Option<String>,
    pub codec: Option<String>,
    pub hdr: Option<String>,
    pub audio: Option<String>,
    pub language: Option<String>,
    pub base_score: i32,
    pub profile_score: i32,
    pub match_score: i32,
    pub accepted: bool,
    pub reasons: Vec<String>,
    pub rejection_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchFailure {
    pub indexer_id: String,
    pub indexer_name: String,
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct SearchContext {
    pub keywords: String,
    pub tmdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub media_type: String,
}

#[derive(Debug, Clone)]
struct Definition {
    id: String,
    name: String,
    base_url: String,
    yaml: Value,
    config: JsonMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default)]
struct TemplateContext {
    keywords: String,
    config: JsonMap<String, JsonValue>,
    query: HashMap<String, String>,
    result: HashMap<String, String>,
    categories: Vec<String>,
}

pub async fn search_indexer(
    state: &AppState,
    indexer_id: &str,
    ctx: &SearchContext,
) -> Result<Vec<ReleaseResult>, String> {
    let def = load_definition(state, indexer_id).await?;
    let client = build_authenticated_client(&def).await?;

    let search = map_get(&def.yaml, "search").ok_or("La definición no contiene search")?;
    let rows = map_get(search, "rows").ok_or("search.rows no existe")?;
    let fields = map_get(search, "fields").ok_or("search.fields no existe")?;

    let mut template = TemplateContext {
        keywords: ctx.keywords.clone(),
        config: def.config.clone(),
        query: HashMap::from([
            ("TMDBID".into(), ctx.tmdb_id.clone().unwrap_or_default()),
            ("IMDBID".into(), ctx.imdb_id.clone().unwrap_or_default()),
        ]),
        result: HashMap::new(),
        categories: category_ids_for_media(&def.yaml, &ctx.media_type),
    };

    // Cardigann definitions can normalize the query before it is inserted in
    // paths/inputs (for example TPB lowercases and YTS removes punctuation).
    template.keywords = apply_filters_ctx(
        template.keywords.clone(),
        map_get(search, "keywordsfilters"),
        &template,
    );

    let paths = search_paths(search);
    if paths.is_empty() {
        return Err("La definición no contiene search.path ni search.paths".into());
    }

    let mut output = Vec::new();

    for path_block in paths.into_iter().take(4) {
        let raw_path = yaml_string(map_get(&path_block, "path")).unwrap_or_default();
        let path = render_template(&raw_path, &template);
        let url = absolute_url(&def.base_url, &path)?;

        let method = yaml_string(map_get(&path_block, "method"))
            .unwrap_or_else(|| "get".into())
            .to_lowercase();

        let mut inputs = yaml_mapping_strings(map_get(search, "inputs"), &template);
        let inherited = map_get(&path_block, "inheritinputs")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        if !inherited {
            inputs.clear();
        }
        inputs.extend(yaml_mapping_strings(
            map_get(&path_block, "inputs"),
            &template,
        ));

        let headers = rendered_headers(map_get(search, "headers"), &template)
            .into_iter()
            .chain(rendered_headers(map_get(&path_block, "headers"), &template))
            .collect::<Vec<_>>();

        let response = if method == "post" {
            let mut req = client.post(&url).form(
                &inputs
                    .iter()
                    .filter(|(k, _)| k != "$raw")
                    .cloned()
                    .collect::<Vec<_>>(),
            );
            for (k, v) in &headers {
                req = req.header(k, v);
            }
            req.send().await
        } else {
            let mut parsed = Url::parse(&url).map_err(|e| e.to_string())?;
            {
                let mut qp = parsed.query_pairs_mut();
                for (k, v) in &inputs {
                    if k != "$raw" && !v.is_empty() {
                        qp.append_pair(k, v);
                    }
                }
            }
            let mut final_url = parsed.to_string();
            if let Some((_, raw)) = inputs.iter().find(|(k, _)| k == "$raw") {
                if !raw.is_empty() {
                    let sep = if final_url.contains('?') { "&" } else { "?" };
                    final_url.push_str(sep);
                    final_url.push_str(raw.trim_start_matches('?').trim_start_matches('&'));
                }
            }
            let mut req = client.get(final_url);
            for (k, v) in &headers {
                req = req.header(k, v);
            }
            req.send().await
        }
        .map_err(|e| format!("Error HTTP en {}: {e}", def.name))?;

        if !response.status().is_success() {
            return Err(format!("{} respondió HTTP {}", def.name, response.status()));
        }

        let response_type = map_get(&path_block, "response")
            .and_then(|r| yaml_string(map_get(r, "type")))
            .or_else(|| map_get(search, "response").and_then(|r| yaml_string(map_get(r, "type"))))
            .unwrap_or_else(|| "html".into())
            .to_lowercase();

        let body = response.text().await.map_err(|e| e.to_string())?;

        if response_type == "json" {
            let json: JsonValue = serde_json::from_str(&body)
                .map_err(|e| format!("{} devolvió JSON no válido: {e}", def.name))?;
            parse_json_rows(&def, &json, rows, fields, &mut template, &mut output)?;
        } else {
            parse_html_rows(&def, &body, rows, fields, &mut template, &mut output)?;
        }
    }

    output.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| b.seeders.unwrap_or(0).cmp(&a.seeders.unwrap_or(0)))
    });
    output.dedup_by(|a, b| a.title == b.title && a.indexer_id == b.indexer_id);
    Ok(output)
}

fn parse_html_rows(
    def: &Definition,
    body: &str,
    rows: &Value,
    fields: &Value,
    template: &mut TemplateContext,
    output: &mut Vec<ReleaseResult>,
) -> Result<(), String> {
    let doc = Html::parse_document(body);
    let row_selector_raw =
        yaml_string(map_get(rows, "selector")).ok_or("search.rows.selector no existe")?;
    let row_selector_rendered = render_template(&row_selector_raw, template);
    let row_selector = forgiving_selector(&row_selector_rendered)
        .ok_or_else(|| format!("Selector de filas no compatible: {row_selector_rendered}"))?;

    for row in doc.select(&row_selector).take(100) {
        template.result.clear();
        let mut extracted: HashMap<String, String> = HashMap::new();

        if let Some(map) = fields.as_mapping() {
            for (key, block) in map {
                let Some(field_name) = key.as_str() else {
                    continue;
                };
                let value = extract_field(&row, block, &def.base_url, template, &extracted);
                if let Some(v) = value {
                    extracted.insert(field_name.to_string(), v.clone());
                    template.result.insert(field_name.to_string(), v);
                }
            }
        }
        push_release(def, extracted, output);
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct JsonRow {
    node: JsonValue,
    parent: Option<JsonValue>,
}

fn parse_json_rows(
    def: &Definition,
    root: &JsonValue,
    rows: &Value,
    fields: &Value,
    template: &mut TemplateContext,
    output: &mut Vec<ReleaseResult>,
) -> Result<(), String> {
    let selector_raw =
        yaml_string(map_get(rows, "selector")).ok_or("search.rows.selector no existe")?;
    let selector = render_template(&selector_raw, template);
    let mut selected = json_select_many(root, &selector)
        .ok_or_else(|| format!("Selector JSON de filas no compatible: {selector}"))?
        .into_iter()
        .map(|v| JsonRow {
            node: v.clone(),
            parent: None,
        })
        .collect::<Vec<_>>();

    if let Some(attribute) = yaml_string(map_get(rows, "attribute")) {
        let multiple = map_get(rows, "multiple")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let mut expanded = Vec::new();
        for row in selected {
            let attr = json_get_path(&row.node, &attribute);
            match attr {
                Some(JsonValue::Array(items)) if multiple => {
                    for item in items {
                        expanded.push(JsonRow {
                            node: item.clone(),
                            parent: Some(row.node.clone()),
                        });
                    }
                }
                Some(value) => expanded.push(JsonRow {
                    node: value.clone(),
                    parent: Some(row.node.clone()),
                }),
                None if map_get(rows, "missingAttributeEqualsNoResults")
                    .and_then(Value::as_bool)
                    .unwrap_or(false) => {}
                None => {}
            }
        }
        selected = expanded;
    }

    // A few Cardigann JSON definitions use :has(field:contains(value)) on the
    // root selector. Support the common uploader form without trying to turn
    // JSON selectors into CSS.
    if let Some((field, wanted)) = json_has_contains_filter(&selector) {
        selected.retain(|row| {
            json_get_path(&row.node, &field)
                .and_then(json_scalar_string)
                .map(|v| v.contains(&wanted))
                .unwrap_or(false)
        });
    }

    let selected_count = selected.len();
    let before = output.len();
    for row in selected.into_iter().take(150) {
        template.result.clear();
        let mut extracted: HashMap<String, String> = HashMap::new();
        if let Some(map) = fields.as_mapping() {
            for (key, block) in map {
                let Some(field_name) = key.as_str() else {
                    continue;
                };
                if let Some(v) =
                    extract_json_field(&row, block, &def.base_url, template, &extracted)
                {
                    extracted.insert(field_name.to_string(), v.clone());
                    template.result.insert(field_name.to_string(), v);
                }
            }
        }
        push_release(def, extracted, output);
    }
    eprintln!(
        "[INDEXER] {} · JSON rows={} releases={}",
        def.name,
        selected_count,
        output.len().saturating_sub(before)
    );
    Ok(())
}

fn push_release(
    def: &Definition,
    mut extracted: HashMap<String, String>,
    output: &mut Vec<ReleaseResult>,
) {
    let title = pick_field(&extracted, &["title", "title_default", "name"]);
    if title.as_deref().unwrap_or("").trim().is_empty() {
        return;
    }
    let title = title.unwrap();

    let details_url = pick_field(&extracted, &["details", "comments"])
        .and_then(|v| make_absolute_maybe(&def.base_url, &v));

    // JSON APIs such as TPB often expose only an infohash. Turn it into a
    // magnet so qBittorrent can receive the result directly.
    if !extracted.contains_key("magnet") && !extracted.contains_key("download") {
        if let Some(hash) = pick_field(&extracted, &["infohash", "info_hash"]) {
            if !hash.trim().is_empty() {
                extracted.insert(
                    "magnet".into(),
                    format!(
                        "magnet:?xt=urn:btih:{}&dn={}",
                        hash.trim(),
                        urlencoding::encode(&title)
                    ),
                );
            }
        }
    }

    let download_url = pick_field(&extracted, &["magnet", "download"])
        .and_then(|v| make_absolute_maybe(&def.base_url, &v));
    let size_bytes = pick_field(&extracted, &["size"]).and_then(|v| parse_size(&v));
    let seeders = pick_field(&extracted, &["seeders"]).and_then(|v| parse_int(&v));
    let leechers = pick_field(&extracted, &["leechers"]).and_then(|v| parse_int(&v));
    let category = pick_field(&extracted, &["categorydesc", "category"]);
    let published = pick_field(&extracted, &["date"]);

    let parsed = releases::parse(&title);
    output.push(ReleaseResult {
        indexer_id: def.id.clone(),
        indexer_name: def.name.clone(),
        title,
        details_url,
        download_url,
        size_bytes,
        seeders,
        leechers,
        category,
        published,
        score: parsed.score + seeders.unwrap_or(0).min(50) as i32,
        resolution: parsed.resolution,
        source: parsed.source,
        codec: parsed.codec,
        hdr: parsed.hdr,
        audio: parsed.audio,
        language: parsed.language,
        base_score: parsed.score,
        profile_score: 0,
        match_score: 0,
        accepted: true,
        reasons: Vec::new(),
        rejection_reasons: Vec::new(),
    });
}

pub async fn test_indexer(state: &AppState, indexer_id: &str) -> Result<String, String> {
    let def = load_definition(state, indexer_id).await?;
    let client = build_authenticated_client(&def).await?;

    if let Some(login) = map_get(&def.yaml, "login") {
        if let Some(test) = map_get(login, "test") {
            let path = yaml_string(map_get(test, "path")).unwrap_or_default();
            let url = absolute_url(
                &def.base_url,
                &render_template(
                    &path,
                    &TemplateContext {
                        config: def.config.clone(),
                        ..Default::default()
                    },
                ),
            )?;
            let response = client.get(url).send().await.map_err(|e| e.to_string())?;
            if !response.status().is_success() {
                return Err(format!("HTTP {}", response.status()));
            }
            if let Some(sel) = yaml_string(map_get(test, "selector")) {
                let html = response.text().await.map_err(|e| e.to_string())?;
                let doc = Html::parse_document(&html);
                let selector = forgiving_selector(&sel)
                    .ok_or_else(|| format!("Selector de test no compatible: {sel}"))?;
                if doc.select(&selector).next().is_none() {
                    return Err("El login respondió, pero el selector de prueba no apareció".into());
                }
            }
            return Ok(format!("{} · login OK", def.name));
        }
    }

    let response = client
        .get(&def.base_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if response.status().is_success() {
        Ok(format!("{} · HTTP {}", def.name, response.status()))
    } else {
        Err(format!("{} respondió HTTP {}", def.name, response.status()))
    }
}

pub async fn fetch_release_bytes_or_url(
    state: &AppState,
    indexer_id: &str,
    direct_url: Option<&str>,
    details_url: Option<&str>,
) -> Result<GrabPayload, String> {
    if let Some(url) = direct_url.filter(|x| !x.trim().is_empty()) {
        if url.starts_with("magnet:") {
            return Ok(GrabPayload::Url(url.to_string()));
        }
    }

    let def = load_definition(state, indexer_id).await?;
    let client = build_authenticated_client(&def).await?;

    let mut candidate = direct_url.map(str::to_string);

    if candidate.is_none() {
        if let Some(details) = details_url {
            if let Some(download) = map_get(&def.yaml, "download") {
                let response = client
                    .get(details)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                if !response.status().is_success() {
                    return Err(format!("Página de detalle HTTP {}", response.status()));
                }
                let body = response.text().await.map_err(|e| e.to_string())?;
                let doc = Html::parse_document(&body);

                if let Some(selectors) = map_get(download, "selectors").and_then(Value::as_sequence)
                {
                    for block in selectors {
                        let Some(sel_raw) = yaml_string(map_get(block, "selector")) else {
                            continue;
                        };
                        let ctx = TemplateContext {
                            config: def.config.clone(),
                            ..Default::default()
                        };
                        let sel_rendered = render_template(&sel_raw, &ctx);
                        let Some(sel) = forgiving_selector(&sel_rendered) else {
                            continue;
                        };
                        if let Some(el) = doc.select(&sel).next() {
                            let attr = yaml_string(map_get(block, "attribute"))
                                .unwrap_or_else(|| "href".into());
                            if let Some(v) = el.value().attr(&attr) {
                                let mut found = v.to_string();
                                found = apply_filters(found, map_get(block, "filters"));
                                candidate = make_absolute_maybe(&def.base_url, &found);
                                if candidate.is_some() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let Some(url) = candidate else {
        return Err("La definición no proporcionó un enlace descargable".into());
    };

    if url.starts_with("magnet:") {
        return Ok(GrabPayload::Url(url));
    }

    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "La descarga del .torrent devolvió HTTP {}",
            response.status()
        ));
    }
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    Ok(GrabPayload::Torrent(bytes.to_vec()))
}

pub enum GrabPayload {
    Url(String),
    Torrent(Vec<u8>),
}

async fn load_definition(state: &AppState, indexer_id: &str) -> Result<Definition, String> {
    let custom = settings::get_value(&state.db, "paths.custom_indexers")
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_else(settings::default_custom_indexers_path);
    let upstream = settings::get_value(&state.db, "paths.upstream_indexers")
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_else(settings::default_upstream_indexers_path);

    let mut chosen: Option<(PathBuf, Value)> = None;
    for folder in [PathBuf::from(custom), PathBuf::from(upstream)] {
        if !folder.exists() {
            continue;
        }
        let mut rd = tokio::fs::read_dir(folder)
            .await
            .map_err(|e| e.to_string())?;
        while let Some(entry) = rd.next_entry().await.map_err(|e| e.to_string())? {
            let path = entry.path();
            if !is_yaml(&path) {
                continue;
            }
            let raw = tokio::fs::read_to_string(&path)
                .await
                .map_err(|e| e.to_string())?;
            let yaml: Value = match serde_yaml::from_str(&raw) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if yaml_string(map_get(&yaml, "id")).as_deref() == Some(indexer_id) {
                chosen = Some((path, yaml));
                break;
            }
        }
        if chosen.is_some() {
            break;
        }
    }

    let (_path, yaml) =
        chosen.ok_or_else(|| format!("No se encontró la definición {indexer_id}"))?;
    let name = yaml_string(map_get(&yaml, "name")).unwrap_or_else(|| indexer_id.into());
    let base_url = yaml_string_list(map_get(&yaml, "links"))
        .into_iter()
        .next()
        .ok_or("La definición no tiene links")?;

    let config_json: Option<String> =
        sqlx::query_scalar("SELECT config_json FROM indexer_configs WHERE indexer_id=?")
            .bind(indexer_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| e.to_string())?;

    let mut config = config_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<JsonValue>(s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    if let Some(settings_list) = map_get(&yaml, "settings").and_then(Value::as_sequence) {
        for item in settings_list {
            let Some(name) = yaml_string(map_get(item, "name")) else {
                continue;
            };
            if config.contains_key(&name) {
                continue;
            }
            if let Some(default) = map_get(item, "default") {
                config.insert(name, yaml_to_json(default));
            }
        }
    }
    // Prowlarr/Cardigann exposes the selected site link as .Config.sitelink.
    config
        .entry("sitelink")
        .or_insert_with(|| JsonValue::String(base_url.clone()));

    Ok(Definition {
        id: indexer_id.into(),
        name,
        base_url,
        yaml,
        config,
    })
}

async fn build_authenticated_client(def: &Definition) -> Result<Client, String> {
    let mut default_headers = HeaderMap::new();
    let ctx = TemplateContext {
        config: def.config.clone(),
        ..Default::default()
    };

    if let Some(login) = map_get(&def.yaml, "login") {
        if yaml_string(map_get(login, "method")).as_deref() == Some("cookie") {
            let inputs = yaml_mapping_strings(map_get(login, "inputs"), &ctx);
            if let Some((_, cookie)) = inputs.iter().find(|(k, _)| k == "cookie") {
                if !cookie.trim().is_empty() {
                    default_headers.insert(
                        COOKIE,
                        HeaderValue::from_str(cookie).map_err(|e| e.to_string())?,
                    );
                }
            }
        }
        for (k, v) in rendered_headers(map_get(login, "headers"), &ctx) {
            if let (Ok(name), Ok(value)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(&v),
            ) {
                default_headers.insert(name, value);
            }
        }
    }

    let client = Client::builder()
        .cookie_store(true)
        .default_headers(default_headers)
        .timeout(Duration::from_secs(18))
        .user_agent("Mozilla/5.0 (compatible; Oberiz/0.4)")
        .build()
        .map_err(|e| e.to_string())?;

    if let Some(login) = map_get(&def.yaml, "login") {
        let method = yaml_string(map_get(login, "method"))
            .unwrap_or_default()
            .to_lowercase();
        if matches!(method.as_str(), "form" | "post" | "get" | "oneurl") {
            let path = yaml_string(map_get(login, "submitpath"))
                .or_else(|| yaml_string(map_get(login, "path")))
                .unwrap_or_default();
            if !path.is_empty() {
                let url = absolute_url(&def.base_url, &render_template(&path, &ctx))?;
                let inputs = yaml_mapping_strings(map_get(login, "inputs"), &ctx);
                let response = if method == "get" {
                    client.get(url).query(&inputs).send().await
                } else {
                    client.post(url).form(&inputs).send().await
                }
                .map_err(|e| e.to_string())?;
                if !response.status().is_success() {
                    return Err(format!("Login HTTP {}", response.status()));
                }
            }
        }
    }

    Ok(client)
}

fn extract_field(
    row: &ElementRef<'_>,
    block: &Value,
    base_url: &str,
    ctx: &TemplateContext,
    extracted: &HashMap<String, String>,
) -> Option<String> {
    let mut local_ctx = ctx.clone();
    local_ctx.result = extracted.clone();

    let mut value = if let Some(text) = map_get(block, "text") {
        yaml_string(Some(text)).map(|v| render_template(&v, &local_ctx))
    } else if let Some(selector_raw) = yaml_string(map_get(block, "selector")) {
        let selector_rendered = render_template(&selector_raw, &local_ctx);
        let selector = forgiving_selector(&selector_rendered)?;
        let el = row.select(&selector).next()?;
        if let Some(attr) = yaml_string(map_get(block, "attribute")) {
            el.value().attr(&attr).map(str::to_string)
        } else {
            Some(el.text().collect::<Vec<_>>().join(" "))
        }
    } else {
        None
    };

    if value.is_none() {
        if let Some(default) = yaml_string(map_get(block, "default")) {
            value = Some(render_template(&default, &local_ctx));
        }
    }

    let mut value = value?;
    value = apply_filters_ctx(value, map_get(block, "filters"), &local_ctx);
    if value.starts_with('/') {
        value = absolute_url(base_url, &value).unwrap_or(value);
    }
    Some(value.trim().to_string())
}

fn apply_filters(value: String, filters: Option<&Value>) -> String {
    apply_filters_ctx(value, filters, &TemplateContext::default())
}

fn apply_filters_ctx(mut value: String, filters: Option<&Value>, ctx: &TemplateContext) -> String {
    let Some(seq) = filters.and_then(Value::as_sequence) else {
        return value;
    };
    for filter in seq {
        let name = yaml_string(map_get(filter, "name")).unwrap_or_default();
        let args = filter_args(map_get(filter, "args"))
            .into_iter()
            .map(|arg| render_template(&arg, ctx))
            .collect::<Vec<_>>();
        match name.as_str() {
            "trim" => value = value.trim().to_string(),
            "replace" if args.len() >= 2 => value = value.replace(&args[0], &args[1]),
            "re_replace" if args.len() >= 2 => {
                if let Ok(re) = Regex::new(&args[0]) {
                    value = re.replace_all(&value, args[1].as_str()).to_string();
                }
            }
            "regexp" if !args.is_empty() => {
                if let Ok(re) = Regex::new(&args[0]) {
                    if let Some(c) = re.captures(&value) {
                        value = c
                            .get(1)
                            .or_else(|| c.get(0))
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();
                    }
                }
            }
            "split" if args.len() >= 2 => {
                if let Ok(i) = args[1].parse::<usize>() {
                    value = value.split(&args[0]).nth(i).unwrap_or("").to_string();
                }
            }
            "prepend" if !args.is_empty() => value = format!("{}{}", args[0], value),
            "append" if !args.is_empty() => value.push_str(&args[0]),
            "tolower" => value = value.to_lowercase(),
            "toupper" => value = value.to_uppercase(),
            "urldecode" => {
                value = urlencoding::decode(&value)
                    .map(|v| v.to_string())
                    .unwrap_or(value)
            }
            "urlencode" => value = urlencoding::encode(&value).to_string(),
            "htmldecode" => value = html_escape::decode_html_entities(&value).to_string(),
            _ => {}
        }
    }
    value
}

fn extract_json_field(
    row: &JsonRow,
    block: &Value,
    base_url: &str,
    ctx: &TemplateContext,
    extracted: &HashMap<String, String>,
) -> Option<String> {
    let mut local_ctx = ctx.clone();
    local_ctx.result = extracted.clone();

    let mut value = if let Some(text) = map_get(block, "text") {
        yaml_string(Some(text)).map(|v| render_template(&v, &local_ctx))
    } else if let Some(selector_raw) = yaml_string(map_get(block, "selector")) {
        let selector = render_template(&selector_raw, &local_ctx);
        let selected = if let Some(rest) = selector.strip_prefix("..") {
            row.parent.as_ref().and_then(|p| json_get_path(p, rest))
        } else {
            json_get_path(&row.node, selector.trim_start_matches("$."))
        };
        selected.and_then(json_scalar_string)
    } else {
        None
    };

    if let Some(case_map) = map_get(block, "case").and_then(Value::as_mapping) {
        if let Some(current) = value.as_deref() {
            let mapped = case_map
                .get(Value::String(current.to_string()))
                .or_else(|| case_map.get(Value::String("*".to_string())))
                .and_then(|v| yaml_string(Some(v)));
            if mapped.is_some() {
                value = mapped;
            }
        }
    }

    if value.is_none() {
        if let Some(default) = yaml_string(map_get(block, "default")) {
            value = Some(render_template(&default, &local_ctx));
        }
    }

    let mut value = value?;
    value = apply_filters_ctx(value, map_get(block, "filters"), &local_ctx);
    if value.starts_with('/') {
        value = absolute_url(base_url, &value).unwrap_or(value);
    }
    Some(value.trim().to_string())
}

fn json_get_path<'a>(root: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    let path = path.trim().trim_start_matches("$.");
    if path.is_empty() || path == "$" {
        return Some(root);
    }
    let mut cur = root;
    for part in path.split('.').filter(|p| !p.is_empty()) {
        cur = match cur {
            JsonValue::Object(map) => map.get(part)?,
            JsonValue::Array(arr) => {
                let idx: usize = part.trim_matches(|c| c == '[' || c == ']').parse().ok()?;
                arr.get(idx)?
            }
            _ => return None,
        };
    }
    Some(cur)
}

fn json_select_many<'a>(root: &'a JsonValue, selector: &str) -> Option<Vec<&'a JsonValue>> {
    let base = selector.split(":has(").next().unwrap_or(selector).trim();
    let selected = json_get_path(root, base)?;
    match selected {
        JsonValue::Array(items) => Some(items.iter().collect()),
        other => Some(vec![other]),
    }
}

fn json_scalar_string(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::Null => None,
        JsonValue::String(s) => Some(s.clone()),
        JsonValue::Bool(b) => Some(b.to_string()),
        JsonValue::Number(n) => Some(n.to_string()),
        JsonValue::Array(arr) => Some(
            arr.iter()
                .filter_map(json_scalar_string)
                .collect::<Vec<_>>()
                .join(","),
        ),
        JsonValue::Object(_) => None,
    }
}

fn json_has_contains_filter(selector: &str) -> Option<(String, String)> {
    let re = Regex::new(r#":has\(([^:()]+):contains\(([^()]*)\)\)\)"#).ok()?;
    let caps = re.captures(selector)?;
    Some((
        caps.get(1)?.as_str().trim().to_string(),
        caps.get(2)?.as_str().trim().to_string(),
    ))
}

fn render_template(input: &str, ctx: &TemplateContext) -> String {
    let mut s = render_conditionals(input, ctx);
    let range_re =
        Regex::new(r#"\{\{\s*range\s+\.Categories\s*\}\}([\s\S]*?)\{\{\s*end\s*\}\}"#).unwrap();
    s = range_re
        .replace_all(&s, |caps: &regex::Captures| {
            ctx.categories
                .iter()
                .map(|cat| caps[1].replace("{{.}}", cat))
                .collect::<Vec<_>>()
                .join("")
        })
        .to_string();

    let token_re = Regex::new(r#"\{\{\s*([^{}]+?)\s*\}\}"#).unwrap();
    token_re
        .replace_all(&s, |caps: &regex::Captures| {
            resolve_token(caps[1].trim(), ctx)
        })
        .to_string()
}

fn render_conditionals(input: &str, ctx: &TemplateContext) -> String {
    let mut out = input.to_string();
    for _ in 0..12 {
        let Some(start) = out.rfind("{{ if ") else {
            break;
        };
        let rest = &out[start..];
        let Some(tag_end_rel) = rest.find("}}") else {
            break;
        };
        let expr = rest[5..tag_end_rel].trim();
        let body_start = start + tag_end_rel + 2;
        let mut depth = 1usize;
        let mut cursor = body_start;
        let mut else_pos = None;
        let mut end_pos = None;
        while cursor < out.len() {
            let Some(rel) = out[cursor..].find("{{") else {
                break;
            };
            let pos = cursor + rel;
            let Some(close_rel) = out[pos..].find("}}") else {
                break;
            };
            let tag = out[pos + 2..pos + close_rel].trim();
            if tag.starts_with("if ") {
                depth += 1;
            } else if tag == "end" {
                depth -= 1;
                if depth == 0 {
                    end_pos = Some((pos, pos + close_rel + 2));
                    break;
                }
            } else if tag == "else" && depth == 1 {
                else_pos = Some((pos, pos + close_rel + 2));
            }
            cursor = pos + close_rel + 2;
        }
        let Some((end_start, end_after)) = end_pos else {
            break;
        };
        let truthy = eval_expr(expr, ctx);
        let replacement = if let Some((else_start, else_after)) = else_pos {
            if truthy {
                &out[body_start..else_start]
            } else {
                &out[else_after..end_start]
            }
        } else if truthy {
            &out[body_start..end_start]
        } else {
            ""
        };
        out = format!("{}{}{}", &out[..start], replacement, &out[end_after..]);
    }
    out
}

fn eval_expr(expr: &str, ctx: &TemplateContext) -> bool {
    let e = expr.trim().trim_matches(|c| c == '(' || c == ')').trim();
    if let Some(rest) = e.strip_prefix("not ") {
        return !eval_expr(rest, ctx);
    }
    if let Some(rest) = e.strip_prefix("and ") {
        return split_expr_args(rest).iter().all(|x| eval_expr(x, ctx));
    }
    if let Some(rest) = e.strip_prefix("or ") {
        return split_expr_args(rest).iter().any(|x| eval_expr(x, ctx));
    }
    if let Some(rest) = e.strip_prefix("eq ") {
        let args = split_expr_args(rest);
        return args.len() >= 2 && resolve_token(&args[0], ctx) == resolve_token(&args[1], ctx);
    }
    let v = resolve_token(e, ctx);
    !v.is_empty() && v != "false" && v != "0"
}

fn split_expr_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut quote = false;
    for c in input.chars() {
        match c {
            '"' => {
                quote = !quote;
                cur.push(c)
            }
            '(' if !quote => {
                depth += 1;
                cur.push(c)
            }
            ')' if !quote => {
                depth -= 1;
                cur.push(c)
            }
            ' ' if !quote && depth == 0 => {
                if !cur.trim().is_empty() {
                    args.push(cur.trim().to_string());
                    cur.clear();
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        args.push(cur.trim().to_string())
    }
    args
}

fn resolve_token(token: &str, ctx: &TemplateContext) -> String {
    let raw = token.trim();
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        return raw[1..raw.len() - 1].to_string();
    }
    let t = raw;
    if t == ".Keywords" {
        return ctx.keywords.clone();
    }
    if let Some(rest) = t.strip_prefix("join .Categories") {
        let sep = rest.trim().trim_matches('"');
        return ctx.categories.join(sep);
    }
    if t == ".True" {
        return "true".into();
    }
    if t == ".False" {
        return "false".into();
    }
    if t == "." {
        return String::new();
    }
    if let Some(key) = t.strip_prefix(".Config.") {
        return json_value_string(ctx.config.get(key)).unwrap_or_default();
    }
    if let Some(key) = t.strip_prefix(".Query.") {
        return ctx.query.get(key).cloned().unwrap_or_default();
    }
    if let Some(key) = t.strip_prefix(".Result.") {
        return ctx.result.get(key).cloned().unwrap_or_default();
    }
    t.to_string()
}

fn search_paths(search: &Value) -> Vec<Value> {
    if let Some(seq) = map_get(search, "paths").and_then(Value::as_sequence) {
        return seq.clone();
    }
    if let Some(path) = map_get(search, "path") {
        let mut map = serde_yaml::Mapping::new();
        map.insert(Value::String("path".into()), path.clone());
        return vec![Value::Mapping(map)];
    }
    vec![]
}
fn category_ids_for_media(root: &Value, media_type: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(seq) = map_get(
        map_get(root, "caps").unwrap_or(&Value::Null),
        "categorymappings",
    )
    .and_then(Value::as_sequence)
    {
        for row in seq {
            let cat = yaml_string(map_get(row, "cat"))
                .unwrap_or_default()
                .to_lowercase();
            let ok = if media_type == "series" {
                cat.starts_with("tv")
            } else {
                cat.starts_with("movies")
            };
            if ok {
                if let Some(id) = yaml_string(map_get(row, "id")) {
                    out.push(id);
                }
            }
        }
    }
    out
}
fn forgiving_selector(raw: &str) -> Option<Selector> {
    if let Ok(s) = Selector::parse(raw) {
        return Some(s);
    }
    let mut cleaned = raw.to_string();
    if let Ok(re) = Regex::new(r#":contains\([^)]*\)"#) {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }
    if let Ok(re) = Regex::new(r#":has\([^)]*\)"#) {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }
    Selector::parse(&cleaned).ok()
}
fn parse_size(raw: &str) -> Option<i64> {
    let trimmed = raw.trim();
    if let Ok(bytes) = trimmed.parse::<i64>() {
        return Some(bytes);
    }
    let normalized = trimmed.replace(',', ".").to_uppercase();
    let re = Regex::new(r#"([0-9]+(?:\.[0-9]+)?)\s*(B|KB|KIB|MB|MIB|GB|GIB|TB|TIB)"#).ok()?;
    let c = re.captures(&normalized)?;
    let n: f64 = c.get(1)?.as_str().parse().ok()?;
    let unit = c.get(2)?.as_str();
    let mult = match unit {
        "KB" | "KIB" => 1024f64,
        "MB" | "MIB" => 1024f64.powi(2),
        "GB" | "GIB" => 1024f64.powi(3),
        "TB" | "TIB" => 1024f64.powi(4),
        _ => 1.0,
    };
    Some((n * mult) as i64)
}
fn parse_int(raw: &str) -> Option<i64> {
    let digits = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-')
        .collect::<String>();
    digits.parse().ok()
}
fn pick_field(map: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = map.get(*k).filter(|x| !x.trim().is_empty()) {
            return Some(v.clone());
        }
    }
    None
}
fn make_absolute_maybe(base: &str, value: &str) -> Option<String> {
    if value.starts_with("magnet:") || value.starts_with("http://") || value.starts_with("https://")
    {
        Some(value.into())
    } else {
        absolute_url(base, value).ok()
    }
}
fn absolute_url(base: &str, path: &str) -> Result<String, String> {
    if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("magnet:") {
        return Ok(path.into());
    }
    Url::parse(base)
        .map_err(|e| e.to_string())?
        .join(path)
        .map(|u| u.to_string())
        .map_err(|e| e.to_string())
}
fn yaml_mapping_strings(value: Option<&Value>, ctx: &TemplateContext) -> Vec<(String, String)> {
    let Some(map) = value.and_then(Value::as_mapping) else {
        return vec![];
    };
    map.iter()
        .filter_map(|(k, v)| {
            let key = k.as_str()?.to_string();
            let val = yaml_string(Some(v)).unwrap_or_default();
            Some((key, render_template(&val, ctx)))
        })
        .collect()
}
fn rendered_headers(value: Option<&Value>, ctx: &TemplateContext) -> Vec<(String, String)> {
    let Some(map) = value.and_then(Value::as_mapping) else {
        return vec![];
    };
    let mut out = Vec::new();
    for (k, v) in map {
        let Some(key) = k.as_str() else { continue };
        if let Some(seq) = v.as_sequence() {
            for item in seq {
                if let Some(s) = yaml_string(Some(item)) {
                    out.push((key.into(), render_template(&s, ctx)))
                }
            }
        } else if let Some(s) = yaml_string(Some(v)) {
            out.push((key.into(), render_template(&s, ctx)))
        }
    }
    out
}
fn filter_args(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Sequence(seq)) => seq.iter().filter_map(|v| yaml_string(Some(v))).collect(),
        Some(v) => yaml_string(Some(v)).into_iter().collect(),
        None => vec![],
    }
}
fn yaml_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Null => JsonValue::Null,
        Value::Bool(x) => JsonValue::Bool(*x),
        Value::Number(n) => n
            .as_i64()
            .map(JsonValue::from)
            .or_else(|| n.as_f64().map(JsonValue::from))
            .unwrap_or(JsonValue::Null),
        Value::String(s) => JsonValue::String(s.clone()),
        Value::Sequence(seq) => JsonValue::Array(seq.iter().map(yaml_to_json).collect()),
        Value::Mapping(_) => JsonValue::Null,
        _ => JsonValue::Null,
    }
}
fn json_value_string(v: Option<&JsonValue>) -> Option<String> {
    match v? {
        JsonValue::String(s) => Some(s.clone()),
        JsonValue::Bool(b) => Some(b.to_string()),
        JsonValue::Number(n) => Some(n.to_string()),
        _ => None,
    }
}
fn yaml_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}
fn yaml_string_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_sequence)
        .map(|s| s.iter().filter_map(|v| yaml_string(Some(v))).collect())
        .unwrap_or_default()
}
fn map_get<'a>(root: &'a Value, key: &str) -> Option<&'a Value> {
    root.as_mapping()?.get(Value::String(key.to_string()))
}
fn is_yaml(path: &Path) -> bool {
    path.extension()
        .and_then(|x| x.to_str())
        .map(|x| x.eq_ignore_ascii_case("yml") || x.eq_ignore_ascii_case("yaml"))
        .unwrap_or(false)
}
