//! Native Windows SMB sessions shared by browsing, scanning and importing.
//! No drive mapping, credentials in command lines, or forced disconnections.
use crate::{AppState, settings};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
const KEY: &str = "storage.smb.connections";
type Error = (StatusCode, String);
#[derive(Clone, Serialize, Deserialize)]
struct Connection {
    server: String,
    username: String,
    secret: Vec<u8>,
    shares: Vec<String>,
}
#[derive(Serialize)]
pub struct Summary {
    server: String,
    username: String,
    shares: Vec<String>,
    password_set: bool,
}
#[derive(Deserialize)]
pub struct Request {
    pub server: String,
    pub username: String,
    pub password: Option<String>,
    pub share: Option<String>,
}
fn host(raw: &str) -> Result<String, Error> {
    let raw = raw.trim().trim_start_matches('\\');
    if raw.is_empty()
        || raw.len() > 253
        || !raw
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Enter an SMB hostname or IPv4 address, not a folder or URL".into(),
        ));
    }
    Ok(raw.to_ascii_lowercase())
}
async fn connections(state: &AppState) -> Result<Vec<Connection>, Error> {
    let raw = settings::get_value(&state.db, KEY)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cannot load SMB configuration".into(),
            )
        })?
        .unwrap_or_else(|| "[]".into());
    serde_json::from_str(&raw).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid SMB configuration".into(),
        )
    })
}
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Summary>>, Error> {
    Ok(Json(
        connections(&state)
            .await?
            .into_iter()
            .map(|c| Summary {
                server: c.server,
                username: c.username,
                shares: c.shares,
                password_set: !c.secret.is_empty(),
            })
            .collect(),
    ))
}
pub(crate) async fn roots(state: &AppState) -> Vec<String> {
    connections(state)
        .await
        .unwrap_or_default()
        .into_iter()
        .flat_map(|c| {
            c.shares
                .into_iter()
                .map(move |s| format!(r"\\{}\{}", c.server, s))
        })
        .collect()
}
static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
pub async fn save(
    State(state): State<AppState>,
    Json(request): Json<Request>,
) -> Result<Json<Summary>, Error> {
    let _guard = LOCK.lock().await;
    let server = host(&request.server)?;
    if request.username.len() > 256 || request.username.contains('\0') {
        return Err((StatusCode::BAD_REQUEST, "Invalid SMB username".into()));
    }
    let share = request.share.as_deref().unwrap_or("").trim().to_string();
    if share.len() > 255
        || share.chars().any(|c| matches!(c, '/' | '\\' | '\0'))
        || matches!(share.as_str(), "." | "..")
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Enter a share name, not a path".into(),
        ));
    }
    let mut saved = connections(&state).await?;
    if saved.len() >= 16 && !saved.iter().any(|c| c.server == server) {
        return Err((StatusCode::BAD_REQUEST, "Too many SMB servers".into()));
    }
    let old = saved.iter().find(|c| c.server == server);
    let secret = match request.password.filter(|p| !p.is_empty()) {
        Some(password) => {
            if password.len() > 4096 || password.contains('\0') {
                return Err((StatusCode::BAD_REQUEST, "Invalid SMB password".into()));
            }
            native::protect(password.as_bytes()).map_err(failure)?
        }
        None => old
            .filter(|c| c.username == request.username)
            .map(|c| c.secret.clone())
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "Enter the SMB password".into()))?,
    };
    let mut connection = Connection {
        server,
        username: request.username,
        secret,
        shares: old.map(|c| c.shares.clone()).unwrap_or_default(),
    };
    let candidate = connection.clone();
    let shares = tokio::task::spawn_blocking(move || native::connect(&candidate, &share))
        .await
        .map_err(|_| failure("SMB worker failed".into()))?
        .map_err(failure)?;
    connection.shares = shares;
    saved.retain(|c| c.server != connection.server);
    saved.push(connection.clone());
    settings::set_value(
        &state.db,
        KEY,
        &serde_json::to_string(&saved)
            .map_err(|_| failure("Cannot serialize SMB configuration".into()))?,
    )
    .await
    .map_err(|_| failure("Cannot save SMB configuration".into()))?;
    Ok(Json(Summary {
        server: connection.server,
        username: connection.username,
        shares: connection.shares,
        password_set: true,
    }))
}
fn failure(message: String) -> Error {
    (StatusCode::BAD_GATEWAY, message)
}
pub(crate) async fn restore(state: &AppState) {
    let _guard = LOCK.lock().await;
    let mut saved = connections(state).await.unwrap_or_default();
    let mut changed = false;
    for connection in &mut saved {
        let server = connection.server.clone();
        let candidate = connection.clone();
        let share = connection.shares.first().cloned().unwrap_or_default();
        if let Ok(Ok(shares)) =
            tokio::task::spawn_blocking(move || native::connect(&candidate, &share)).await
        {
            if shares != connection.shares {
                connection.shares = shares;
                changed = true;
            }
        } else {
            tracing::warn!(
                server,
                "SMB connection could not be restored; reconnect in Settings"
            );
        }
    }
    if changed && let Ok(raw) = serde_json::to_string(&saved) {
        let _ = settings::set_value(&state.db, KEY, &raw).await;
    }
}
#[derive(Deserialize)]
pub struct Forget {
    pub server: String,
}
pub async fn forget(
    State(state): State<AppState>,
    Json(request): Json<Forget>,
) -> Result<Json<serde_json::Value>, Error> {
    let _guard = LOCK.lock().await;
    let server = host(&request.server)?;
    let mut saved = connections(&state).await?;
    saved.retain(|c| c.server != server);
    settings::set_value(
        &state.db,
        KEY,
        &serde_json::to_string(&saved)
            .map_err(|_| failure("Cannot serialize SMB configuration".into()))?,
    )
    .await
    .map_err(|_| failure("Cannot save SMB configuration".into()))?;
    Ok(Json(serde_json::json!({"status":"ok"})))
}
#[cfg(not(windows))]
mod native {
    use super::Connection;
    pub fn protect(_: &[u8]) -> Result<Vec<u8>, String> {
        Err("On Linux, mount SMB shares with the OS and select their mount paths".into())
    }
    pub fn connect(_: &Connection, _: &str) -> Result<Vec<String>, String> {
        Err("On Linux, use OS-mounted SMB paths".into())
    }
}
#[cfg(windows)]
mod native {
    use super::Connection;
    use windows_sys::Win32::{
        Foundation::LocalFree,
        NetworkManagement::{NetManagement::NetApiBufferFree, WNet::*},
        Security::Cryptography::*,
        Storage::FileSystem::{NetShareEnum, SHARE_INFO_1},
    };
    fn wide(raw: &str) -> Vec<u16> {
        raw.encode_utf16().chain(Some(0)).collect()
    }
    pub fn protect(raw: &[u8]) -> Result<Vec<u8>, String> {
        crypt(raw, true)
    }
    fn crypt(raw: &[u8], encrypt: bool) -> Result<Vec<u8>, String> {
        let input = CRYPT_INTEGER_BLOB {
            cbData: raw.len() as u32,
            pbData: raw.as_ptr() as *mut u8,
        };
        let mut output: CRYPT_INTEGER_BLOB = unsafe { std::mem::zeroed() };
        let result = unsafe {
            if encrypt {
                CryptProtectData(
                    &input,
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    CRYPTPROTECT_LOCAL_MACHINE | CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output,
                )
            } else {
                CryptUnprotectData(
                    &input,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output,
                )
            }
        };
        if result == 0 {
            return Err("Cannot protect/unprotect SMB password on this Windows installation; re-enter credentials".into());
        }
        let bytes =
            unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
        unsafe {
            LocalFree(output.pbData.cast());
        }
        Ok(bytes)
    }
    pub fn connect(connection: &Connection, share: &str) -> Result<Vec<String>, String> {
        let mut plain = crypt(&connection.secret, false)?;
        let password = std::str::from_utf8(&plain).map_err(|_| "Invalid protected SMB password")?;
        let mut password = wide(password);
        plain.fill(0);
        let remote = wide(&format!(
            r"\\{}\{}",
            connection.server,
            if share.is_empty() { "IPC$" } else { share }
        ));
        let username = wide(&connection.username);
        let resource = NETRESOURCEW {
            dwType: RESOURCETYPE_ANY,
            lpRemoteName: remote.as_ptr() as *mut _,
            ..unsafe { std::mem::zeroed() }
        };
        let status =
            unsafe { WNetAddConnection2W(&resource, password.as_ptr(), username.as_ptr(), 0) };
        password.fill(0);
        if status != 0 {
            return Err(if status == 1219 {
                "An SMB session already uses different credentials. Oberiz will not disconnect it; use the same account or close that session yourself.".into()
            } else {
                format!("SMB authentication failed (Windows error {status})")
            });
        }
        let server = wide(&format!(r"\\{}", connection.server));
        let mut shares = connection.shares.clone();
        if !share.is_empty() {
            shares.push(share.into());
        }
        let mut resume = 0;
        for _ in 0..32 {
            let mut buffer = std::ptr::null_mut();
            let mut count = 0;
            let mut total = 0;
            let status = unsafe {
                NetShareEnum(
                    server.as_ptr(),
                    1,
                    &mut buffer,
                    16384,
                    &mut count,
                    &mut total,
                    &mut resume,
                )
            };
            if !buffer.is_null() {
                if status == 0 || status == 234 {
                    for info in unsafe {
                        std::slice::from_raw_parts(buffer.cast::<SHARE_INFO_1>(), count as usize)
                    } {
                        if info.shi1_type & 0xffff == 0 && !info.shi1_netname.is_null() {
                            let mut len = 0;
                            unsafe {
                                while len < 256 && *info.shi1_netname.add(len) != 0 {
                                    len += 1;
                                }
                                let name = String::from_utf16_lossy(std::slice::from_raw_parts(
                                    info.shi1_netname,
                                    len,
                                ));
                                if !name.ends_with('$') {
                                    shares.push(name);
                                }
                            }
                        }
                    }
                }
                unsafe {
                    NetApiBufferFree(buffer.cast());
                }
            }
            if status != 234 {
                if status != 0 && shares.is_empty() {
                    return Err(format!(
                        "SMB connected, but share listing failed (Windows error {status}); supply a known share name"
                    ));
                }
                break;
            }
        }
        shares.sort();
        shares.dedup();
        Ok(shares)
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn machine_protected_password_round_trip_and_corruption_detection() {
            let input = b"synthetic-password-for-test";
            let protected = protect(input).unwrap();
            assert_ne!(protected, input);
            assert_eq!(crypt(&protected, false).unwrap(), input);
            assert!(crypt(b"invalid-protected-data", false).is_err());
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_server_not_url_or_path() {
        assert_eq!(host(r"\\192.168.12.200").unwrap(), "192.168.12.200");
        for bad in [
            "",
            "http://host",
            "host/share",
            "host\\share",
            "host\0",
            "../host",
        ] {
            assert!(host(bad).is_err());
        }
    }
    #[tokio::test]
    async fn saved_shares_appear_without_opening_them_and_secrets_are_never_returned() {
        use std::sync::Arc;
        use tokio::sync::Mutex;
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
            download_client: Arc::new(crate::download_client::QBittorrentClient),
            torrent_list_cache: Arc::new(Mutex::new(None)),
            grab_lock: Arc::new(Mutex::new(())),
        };
        let saved = Connection {
            server: "fixture.invalid".into(),
            username: "fixture-user".into(),
            secret: vec![1, 2, 3],
            shares: vec!["disk-a".into(), "disk-b".into()],
        };
        settings::set_value(
            &state.db,
            KEY,
            &serde_json::to_string(&vec![saved]).unwrap(),
        )
        .await
        .unwrap();
        let result = list(State(state.clone())).await.unwrap().0;
        let json = serde_json::to_value(result).unwrap();
        assert!(json[0].get("secret").is_none());
        assert!(json[0].get("password").is_none());
        assert_eq!(json[0]["password_set"], true);
        assert_eq!(
            roots(&state).await,
            vec![r"\\fixture.invalid\disk-a", r"\\fixture.invalid\disk-b"]
        );
        let listing = crate::filesystem::list_directories(
            State(state.clone()),
            axum::extract::Query(crate::filesystem::DirectoryQuery { path: None }),
        )
        .await
        .unwrap()
        .0;
        assert!(listing.path.is_empty());
        assert!(
            listing
                .directories
                .iter()
                .any(|entry| entry.path == r"\\fixture.invalid\disk-a")
        );
        let _ = forget(
            State(state.clone()),
            Json(Forget {
                server: "fixture.invalid".into(),
            }),
        )
        .await
        .unwrap();
        assert!(roots(&state).await.is_empty());
        assert_eq!(
            settings::get_value(&state.db, KEY)
                .await
                .unwrap()
                .as_deref(),
            Some("[]")
        );
    }
}
