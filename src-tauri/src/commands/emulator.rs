use std::{collections::HashMap, fs::File, io::Write};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::StoreExt;

use crate::{store, ApiResponse};

#[derive(Deserialize, Debug)]
pub struct ApiEmulator {
    pub id: i32,
    pub name: String,
    pub console: String,
    pub platform: String,
    pub run_command: String,
    pub binary_path: String,
    pub config_files: Vec<String>,
    pub zipped: bool,
    pub md5_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoreEmulator {
    pub is_installed: bool,
    pub binary_path: String,
    pub run_command: String,
    pub config_files: Vec<String>,
    pub zipped: bool,
}

#[tauri::command]
pub async fn download_emulator<R: Runtime>(
    app: AppHandle<R>,
    console: String,
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
    let emulator = emulators.into_iter().next().ok_or("No emulator found")?;

    let mut app_data_dir = store::get_data_dir(&app)?;
    app_data_dir.push("emulators");
    app_data_dir.push(&console);
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

        let exec_name = emulator.run_command.split_whitespace().next().unwrap_or("");
        final_binary_path = app_data_dir.join(exec_name);
    }

    let mut stored_emulators: HashMap<String, StoreEmulator> = store
        .get("emulators")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    stored_emulators.insert(
        console.clone(),
        StoreEmulator {
            is_installed: true,
            binary_path: final_binary_path.to_string_lossy().to_string(),
            run_command: emulator.run_command,
            config_files: emulator.config_files,
            zipped: emulator.zipped,
        },
    );

    store.set(
        "emulators",
        serde_json::to_value(stored_emulators).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;

    Ok(())
}
