use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use tauri::{AppHandle, Runtime};

use crate::{store, ApiResponse};

#[derive(serde::Serialize)]
pub struct SyncResult {
    pub conflict: bool,
    pub latest_version: Option<i32>,
}

#[derive(Deserialize)]
pub struct SaveFileMetadata {
    pub file_name: String,
    pub version_id: i32,
}

#[tauri::command]
pub async fn check_save_status<R: Runtime>(
    app: AppHandle<R>,
    game_id: String,
) -> Result<SyncResult, String> {
    let store = store::get_current_store(&app)?;
    let domain = store
        .get("domain")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("No domain")?;
    let token = store
        .get("token")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("No token")?;

    let last_known_version = store
        .get("sync_versions")
        .and_then(|v| v.get(&game_id).cloned())
        .and_then(|v| v.as_i64().map(|i| i as i32))
        .unwrap_or(0);

    let base_path = store::get_data_dir(&app)?;
    let save_dir = base_path.join("saves").join(&game_id);

    let api_res = match crate::commands::http::perform_backend_request(
        &app,
        reqwest::Method::GET,
        &format!(
            "{}/api/v1/roms/{}/save/latest",
            domain.trim_end_matches('/'),
            game_id
        ),
        Some(&token),
        crate::commands::http::BackendBody::None,
        false,
    )
    .await
    {
        Ok(res) => res.into_json::<ApiResponse<Vec<SaveFileMetadata>>>()?,
        Err(_) => {
            return Ok(SyncResult {
                conflict: false,
                latest_version: None,
            });
        }
    };

    let files = api_res.data.unwrap_or_default();
    let server_v = files.first().map(|f| f.version_id).unwrap_or(0);

    let local_exists = std::fs::read_dir(&save_dir)
        .map(|mut d| d.next().is_some())
        .unwrap_or(false);

    Ok(SyncResult {
        conflict: local_exists && server_v > last_known_version,
        latest_version: Some(server_v),
    })
}

#[tauri::command]
pub async fn download_save_files<R: Runtime>(
    app: AppHandle<R>,
    game_id: String,
) -> Result<(), String> {
    let store = store::get_current_store(&app)?;
    let domain = store
        .get("domain")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("No domain")?;
    let token = store
        .get("token")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("No token")?;

    let base_path = store::get_data_dir(&app)?;
    let save_dir = base_path.join("saves").join(&game_id);
    std::fs::create_dir_all(&save_dir).map_err(|e| e.to_string())?;

    let api_res = crate::commands::http::perform_backend_request(
        &app,
        reqwest::Method::GET,
        &format!(
            "{}/api/v1/roms/{}/save/latest",
            domain.trim_end_matches('/'),
            game_id
        ),
        Some(&token),
        crate::commands::http::BackendBody::None,
        false,
    )
    .await?
    .into_json::<ApiResponse<Vec<SaveFileMetadata>>>()?;

    let files = api_res.data.unwrap_or_default();
    let server_v = files.first().map(|f| f.version_id).unwrap_or(0);

    for file_meta in files {
        let payload = serde_json::json!({ "path": file_meta.file_name });
        let data = crate::commands::http::perform_backend_request(
            &app,
            reqwest::Method::POST,
            &format!(
                "{}/api/v1/roms/{}/save/{}/download",
                domain.trim_end_matches('/'),
                game_id,
                file_meta.version_id
            ),
            Some(&token),
            crate::commands::http::BackendBody::Json(
                serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null),
            ),
            true,
        )
        .await?
        .into_stream()?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

        let save_path = save_dir.join(&file_meta.file_name);
        if let Some(parent) = save_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(save_path, data).map_err(|e| e.to_string())?;
    }

    let mut sync_versions = store
        .get("sync_versions")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    sync_versions.insert(game_id, serde_json::json!(server_v));
    store.set("sync_versions", serde_json::json!(sync_versions));
    let _ = store.save();

    Ok(())
}

pub async fn upload_save_files<R: Runtime>(
    app: AppHandle<R>,
    game_id: &str,
    save_dir: &std::path::PathBuf,
    version_id: i32,
) -> Result<(), String> {
    let store = store::get_current_store(&app)?;
    let domain = store
        .get("domain")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("No domain")?;
    let token = store
        .get("token")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("No token")?;

    let mut files_list = Vec::new();

    if !save_dir.exists() {
        return Ok(());
    }

    let mut stack = vec![save_dir.clone()];
    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            if path.is_file() {
                let rel = path
                    .strip_prefix(save_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace("\\", "/");

                let bytes = match std::fs::read(&path) {
                    Ok(b) => b,
                    Err(_) => continue,
                };

                let digest = md5::compute(&bytes);
                let hash = format!("{:x}", digest);
                let content = general_purpose::STANDARD.encode(&bytes);

                files_list.push(serde_json::json!({
                    "hash": hash,
                    "path": rel,
                    "content": content
                }));
            }
        }
    }

    if !files_list.is_empty() {
        let payload = serde_json::json!({ "files": files_list });
        crate::commands::http::perform_backend_request(
            &app,
            reqwest::Method::POST,
            &format!(
                "{}/api/v1/roms/{}/save/{}",
                domain.trim_end_matches('/'),
                game_id,
                version_id
            ),
            Some(&token),
            crate::commands::http::BackendBody::Json(
                serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null),
            ),
            false,
        )
        .await?;
    }
    Ok(())
}

#[derive(serde::Serialize)]
pub struct GameSaveFile {
    pub path: String,
    pub size: u64,
    pub hash: String,
}

#[tauri::command]
pub fn get_game_save_files<R: Runtime>(
    app: AppHandle<R>,
    game_id: String,
) -> Result<Vec<GameSaveFile>, String> {
    let base_path = store::get_data_dir(&app)?;
    let save_dir = base_path.join("saves").join(&game_id);
    let mut files = Vec::new();

    if !save_dir.exists() {
        return Ok(files);
    }

    let mut stack = vec![save_dir.clone()];

    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            if path.is_file() {
                let rel = path
                    .strip_prefix(&save_dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace("\\", "/");

                let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                let hash = if let Ok(bytes) = std::fs::read(&path) {
                    format!("{:x}", md5::compute(&bytes))
                } else {
                    String::new()
                };

                files.push(GameSaveFile {
                    path: rel,
                    size,
                    hash,
                });
            }
        }
    }

    Ok(files)
}

#[tauri::command]
pub fn open_save_folder<R: Runtime>(app: AppHandle<R>, game_id: String) -> Result<(), String> {
    let base_path = store::get_data_dir(&app)?;
    let save_dir = base_path.join("saves").join(&game_id);

    if !save_dir.exists() {
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&save_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&save_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&save_dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
