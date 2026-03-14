use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use std::collections::HashMap;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::StoreExt;

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
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
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

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/api/v1/roms/{}/save/latest",
            domain.trim_end_matches('/'),
            game_id
        ))
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Ok(SyncResult {
            conflict: false,
            latest_version: None,
        });
    }

    let api_res: ApiResponse<Vec<SaveFileMetadata>> =
        resp.json().await.map_err(|e| e.to_string())?;
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

    let base_path = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let save_dir = base_path.join("saves").join(&game_id);
    std::fs::create_dir_all(&save_dir).map_err(|e| e.to_string())?;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "{}/api/v1/roms/{}/save/latest",
            domain.trim_end_matches('/'),
            game_id
        ))
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let api_res: ApiResponse<Vec<SaveFileMetadata>> =
        resp.json().await.map_err(|e| e.to_string())?;
    let files = api_res.data.unwrap_or_default();
    let server_v = files.first().map(|f| f.version_id).unwrap_or(0);

    for file_meta in files {
        let data = client
            .get(format!(
                "{}/api/v1/roms/{}/save/{}/{}",
                domain.trim_end_matches('/'),
                game_id,
                file_meta.version_id,
                file_meta.file_name
            ))
            .header("Authorization", &token)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;

        std::fs::write(save_dir.join(file_meta.file_name), data).map_err(|e| e.to_string())?;
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

    let mut files_map = HashMap::new();

    for entry in std::fs::read_dir(save_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            let bytes = std::fs::read(entry.path()).map_err(|e| e.to_string())?;
            let b64 = general_purpose::STANDARD.encode(bytes);
            files_map.insert(name, b64);
        }
    }

    if !files_map.is_empty() {
        let payload = serde_json::json!({ "files": files_map });
        let client = reqwest::Client::new();
        let resp = client
            .post(format!(
                "{}/api/v1/roms/{}/save/{}",
                domain.trim_end_matches('/'),
                game_id,
                version_id
            ))
            .header("Authorization", token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Upload failed: {}", resp.status()));
        }
    }
    Ok(())
}
