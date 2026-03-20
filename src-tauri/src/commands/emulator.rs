use std::{collections::HashMap, fs::File, io::Write};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::{store, ApiResponse};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApiEmulator {
    pub id: String,
    pub name: String,
    pub consoles: Vec<String>,
    pub platform: String,
    pub run_command: String,
    pub binary_path: String,
    pub binary_name: Option<String>,
    pub save_path: Option<String>,
    pub config_files: Vec<String>,
    pub zipped: bool,
    pub file_size: i64,
    pub md5_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoreEmulator {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub consoles: Vec<String>,
    pub is_default: bool,
    pub is_installed: bool,
    pub binary_path: String,
    pub run_command: String,
    #[serde(default)]
    pub save_path: Option<String>,
    #[serde(default)]
    pub config_files: Vec<String>,
    #[serde(default)]
    pub zipped: bool,
}

#[tauri::command]
pub async fn fetch_server_emulators<R: Runtime>(
    app: AppHandle<R>,
    console: String,
) -> Result<Vec<ApiEmulator>, String> {
    let store = store::get_current_store(&app)?;

    let domain = store
        .get("domain")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Domain not found")?;
    let token = store
        .get("token")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Token not found")?;

    let platform = std::env::consts::OS;
    let api_url = format!(
        "{}/api/v1/emulators/{}/{}",
        domain.trim_end_matches('/'),
        console,
        platform
    );

    let client = reqwest::Client::new();
    let response_text = client
        .get(&api_url)
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let api_res: ApiResponse<Vec<ApiEmulator>> =
        serde_json::from_str(&response_text).map_err(|e| e.to_string())?;

    Ok(api_res.data.unwrap_or_default())
}

#[tauri::command]
pub async fn fetch_all_server_emulators<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Vec<ApiEmulator>, String> {
    let store = store::get_current_store(&app)?;

    let domain = store
        .get("domain")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Domain not found")?;
    let token = store
        .get("token")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Token not found")?;

    let platform = std::env::consts::OS;
    let api_url = format!("{}/api/v1/emulators/all", domain.trim_end_matches('/'));

    let client = reqwest::Client::new();
    let response_text = client
        .get(&api_url)
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let api_res: ApiResponse<Vec<ApiEmulator>> =
        serde_json::from_str(&response_text).map_err(|e| e.to_string())?;

    let emulators = api_res.data.unwrap_or_default();

    let filtered: Vec<ApiEmulator> = emulators
        .into_iter()
        .filter(|e| e.platform == platform)
        .collect();

    Ok(filtered)
}

#[tauri::command]
pub async fn download_emulator<R: Runtime>(
    app: AppHandle<R>,
    console: String,
    emulator_id: Option<String>,
) -> Result<(), String> {
    let store = store::get_current_store(&app)?;

    let domain = store
        .get("domain")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Domain not found")?;
    let token = store
        .get("token")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Token not found")?;
    let storage_path = store
        .get("storage_path")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Storage path not found")?;

    let platform = std::env::consts::OS;
    let api_url = format!(
        "{}/api/v1/emulators/{}/{}",
        domain.trim_end_matches('/'),
        console,
        platform
    );

    let client = reqwest::Client::new();
    let response_text = client
        .get(&api_url)
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;

    let api_res: ApiResponse<Vec<ApiEmulator>> =
        serde_json::from_str(&response_text).map_err(|e| e.to_string())?;
    let emulators = api_res.data.ok_or("No data")?;

    let emulator = if let Some(id) = emulator_id {
        emulators
            .into_iter()
            .find(|e| e.id == id)
            .ok_or("Emulator not found")?
    } else {
        emulators.into_iter().next().ok_or("No emulator found")?
    };

    let mut app_data_dir = store::get_data_dir(&app)?;
    app_data_dir.push("emulators");
    let console_lowercased = console.to_lowercase();
    app_data_dir.push(&console_lowercased);
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;

    let file_name = emulator
        .binary_path
        .split('/')
        .last()
        .unwrap_or("emulator.bin");
    let local_file_path = app_data_dir.join(file_name);

    let download_url = format!(
        "{}{}/{}",
        domain.trim_end_matches('/'),
        storage_path,
        emulator.binary_path.trim_start_matches('/')
    );

    let response = client
        .get(&download_url)
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let mut stream = response.bytes_stream();
    let mut file = File::create(&local_file_path).map_err(|e| e.to_string())?;

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
    }

    let mut final_binary_path = local_file_path.clone();

    if emulator.zipped {
        let zip_file = File::open(&local_file_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(zip_file).map_err(|e| e.to_string())?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file.enclosed_name() {
                Some(path) => app_data_dir.join(path),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).ok();
                }
            }
        }

        std::fs::remove_file(&local_file_path).ok();

        let exec_name = if let Some(ref custom_name) = emulator.binary_name {
            if custom_name.is_empty() {
                emulator.run_command.split_whitespace().next().unwrap_or("")
            } else {
                custom_name
            }
        } else {
            emulator.run_command.split_whitespace().next().unwrap_or("")
        };
        final_binary_path = app_data_dir.join(exec_name);
    }

    let mut stored_emulators: HashMap<String, StoreEmulator> = store
        .get("emulators")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let server_id = format!("server-{}", emulator.id);
    let is_first = stored_emulators.is_empty();

    if let Some(existing) = stored_emulators.get_mut(&server_id) {
        existing.binary_path = final_binary_path.to_string_lossy().to_string();
        existing.run_command = emulator.run_command.clone();
        existing.save_path = emulator.save_path.clone();
        existing.config_files = emulator.config_files.clone();
        existing.zipped = emulator.zipped;
        existing.is_installed = true;
        for c in &emulator.consoles {
            if !existing.consoles.contains(c) {
                existing.consoles.push(c.clone());
            }
        }
    } else {
        stored_emulators.insert(server_id.clone(), StoreEmulator {
            id: server_id,
            name: emulator.name.clone(),
            consoles: emulator.consoles.clone(),
            is_default: is_first,
            is_installed: true,
            binary_path: final_binary_path.to_string_lossy().to_string(),
            run_command: emulator.run_command,
            save_path: emulator.save_path,
            config_files: emulator.config_files,
            zipped: emulator.zipped,
        });
    }

    store.set(
        "emulators",
        serde_json::to_value(stored_emulators).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;

    Ok(())
}
