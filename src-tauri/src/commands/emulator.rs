use std::{collections::HashMap, fs::File, io::Write};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

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
    pub save_extensions: Vec<String>,
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
    pub binary_path: String,
    pub run_command: String,
    #[serde(default)]
    pub save_path: Option<String>,
    #[serde(default)]
    pub save_extensions: Vec<String>,
    #[serde(default)]
    pub config_files: Vec<String>,
    #[serde(default)]
    pub zipped: bool,
    #[serde(default)]
    pub file_size: i64,
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

    let platform = std::env::consts::OS.to_lowercase();
    let console_upper = console.to_uppercase();
    let api_url = format!(
        "{}/api/v1/emulators/{}/{}",
        domain.trim_end_matches('/'),
        console_upper,
        platform
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&api_url)
        .header("Authorization", &token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let response_text = response.text().await.map_err(|e| e.to_string())?;

    let api_res: ApiResponse<Vec<ApiEmulator>> =
        serde_json::from_str(&response_text).map_err(|e| e.to_string())?;

    let filtered = api_res.data.unwrap_or_default();
    println!(
        "[Rust] Found {} matching emulators for console: {}",
        filtered.len(),
        console_upper
    );

    Ok(filtered)
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

    let platform = std::env::consts::OS.to_lowercase();
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
        .filter(|e| e.platform.to_lowercase() == platform)
        .collect();

    Ok(filtered)
}

#[tauri::command]
pub async fn download_emulator<R: Runtime>(
    app: AppHandle<R>,
    console: String,
    emulator_id: Option<String>,
) -> Result<(), String> {
    let current_store = store::get_current_store(&app)?;
    let global_store = store::get_global_store(&app)?;

    let domain = current_store
        .get("domain")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Domain not found")?;
    let token = current_store
        .get("token")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Token not found")?;
    let storage_path = current_store
        .get("storage_path")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Storage path not found")?;

    let platform = std::env::consts::OS.to_lowercase();
    let console_upper = console.to_uppercase();
    let api_url = format!(
        "{}/api/v1/emulators/{}/{}",
        domain.trim_end_matches('/'),
        console_upper,
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

    println!("Server response received (length: {})", response_text.len());

    let api_res: ApiResponse<Vec<ApiEmulator>> =
        serde_json::from_str(&response_text).map_err(|e| {
            println!(
                "[Rust] Failed to parse JSON in download_emulator: {} | Raw JSON: {}",
                e, response_text
            );
            e.to_string()
        })?;
    let emulators = api_res.data.ok_or_else(|| {
        println!("[Rust] No 'data' field in server response");
        "No data".to_string()
    })?;

    println!("[Rust] Found {} candidate(s) in list", emulators.len());

    let emulator = if let Some(id) = emulator_id {
        emulators.into_iter().find(|e| e.id == id).ok_or_else(|| {
            println!(
                "[Rust] Specified emulator ID '{}' not found in candidate list",
                id
            );
            "Emulator not found".to_string()
        })?
    } else {
        emulators.into_iter().next().ok_or_else(|| {
            println!(
                "[Rust] Empty emulator candidate list for console: {}",
                console
            );
            "No emulator found".to_string()
        })?
    };

    println!(
        "Selected emulator: '{}' (ID: {})",
        emulator.name, emulator.id
    );

    let mut app_data_dir = store::get_data_dir(&app)?;
    app_data_dir.push("emulators");
    app_data_dir.push(console.to_lowercase());
    let emu_name_safe = emulator.name.replace(" ", "_").to_lowercase();
    app_data_dir.push(&emu_name_safe);
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

    let mut stored_emulators: HashMap<String, StoreEmulator> = global_store
        .get("emulators")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let safe_domain = domain
        .replace(|c: char| !c.is_alphanumeric(), "_")
        .to_lowercase();
    let server_id = format!("server-{}-{}", safe_domain, emulator.id);
    let is_first = stored_emulators.is_empty();

    if let Some(existing) = stored_emulators.get_mut(&server_id) {
        existing.binary_path = final_binary_path.to_string_lossy().to_string();
        existing.run_command = emulator.run_command.clone();
        existing.save_path = emulator.save_path.clone();
        existing.save_extensions = emulator.save_extensions.clone();
        existing.config_files = emulator.config_files.clone();
        existing.zipped = emulator.zipped;

        for c in &emulator.consoles {
            if !existing.consoles.contains(c) {
                existing.consoles.push(c.clone());
            }
        }
    } else {
        stored_emulators.insert(
            server_id.clone(),
            StoreEmulator {
                id: server_id,
                name: emulator.name.clone(),
                consoles: emulator.consoles.clone(),
                is_default: is_first,
                binary_path: final_binary_path.to_string_lossy().to_string(),
                run_command: emulator.run_command,
                save_path: emulator.save_path,
                save_extensions: emulator.save_extensions,
                config_files: emulator.config_files,
                zipped: emulator.zipped,
                file_size: emulator.file_size,
            },
        );
    }

    global_store.set(
        "emulators",
        serde_json::to_value(stored_emulators).map_err(|e| e.to_string())?,
    );
    global_store.save().map_err(|e| e.to_string())?;

    Ok(())
}

fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

#[tauri::command]
pub async fn get_emulator_dir_sizes<R: Runtime>(
    app: AppHandle<R>,
) -> Result<HashMap<String, u64>, String> {
    let store = store::get_global_store(&app)?;
    let stored_emulators: HashMap<String, StoreEmulator> = store
        .get("emulators")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let mut sizes: HashMap<String, u64> = HashMap::new();
    for (id, emu) in &stored_emulators {
        let bin_path = std::path::Path::new(&emu.binary_path);
        if let Some(dir) = bin_path.parent() {
            if dir.exists() && dir.is_dir() {
                sizes.insert(id.clone(), dir_size(dir));
            } else if bin_path.exists() {
                sizes.insert(
                    id.clone(),
                    bin_path.metadata().map(|m| m.len()).unwrap_or(0),
                );
            }
        }
    }

    Ok(sizes)
}

#[tauri::command]
pub async fn remove_emulator<R: Runtime>(
    app: AppHandle<R>,
    emulator_id: String,
) -> Result<(), String> {
    let store = store::get_global_store(&app)?;
    let mut stored_emulators: HashMap<String, StoreEmulator> = store
        .get("emulators")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    if let Some(emulator) = stored_emulators.remove(&emulator_id) {
        let bin_path = std::path::Path::new(&emulator.binary_path);
        if let Some(emu_dir) = bin_path.parent() {
            if emu_dir.exists() && emu_dir.is_dir() {
                if emu_dir.to_string_lossy().contains("emulators") {
                    let _ = std::fs::remove_dir_all(emu_dir);
                }
            }
        }

        store.set(
            "emulators",
            serde_json::to_value(stored_emulators).map_err(|e| e.to_string())?,
        );
        store.save().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn migrate_emulator_files<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let store = store::get_global_store(&app)?;
    let mut stored_emulators: HashMap<String, StoreEmulator> = store
        .get("emulators")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let base_app_data = store::get_data_dir(&app)?;
    let mut changed = false;

    for emulator in stored_emulators.values_mut() {
        if emulator.id.starts_with("custom-") {
            continue;
        }

        let old_bin_path = std::path::PathBuf::from(&emulator.binary_path);
        let emu_name_safe = emulator.name.replace(" ", "_").to_lowercase();
        let mut new_dir = base_app_data.clone();
        new_dir.push("emulators");
        let console_folder = emulator
            .consoles
            .first()
            .map(|c| c.to_lowercase())
            .unwrap_or_else(|| "system".to_string());
        new_dir.push(&console_folder);
        new_dir.push(&emu_name_safe);

        if old_bin_path.starts_with(&new_dir) {
            continue;
        }

        if !old_bin_path.exists() {
            continue;
        }

        let _ = std::fs::create_dir_all(&new_dir);

        if let Some(old_dir) = old_bin_path.parent() {
            if old_dir.exists() && old_dir.is_dir() {
                let old_str = old_dir.to_string_lossy();
                if old_str.contains("emulators") || old_str.contains("domains") {
                    if let Ok(entries) = std::fs::read_dir(old_dir) {
                        for entry in entries.flatten() {
                            let fname = entry.file_name();
                            let dest = new_dir.join(fname);
                            let _ = std::fs::rename(entry.path(), dest);
                        }
                    }

                    let bin_name = old_bin_path.file_name().unwrap_or_default();
                    let new_bin_path = new_dir.join(bin_name);
                    emulator.binary_path = new_bin_path.to_string_lossy().to_string();
                    changed = true;

                    let _ = std::fs::remove_dir_all(old_dir);
                }
            }
        }
    }

    if changed {
        store.set(
            "emulators",
            serde_json::to_value(stored_emulators).map_err(|e| e.to_string())?,
        );
        store.save().map_err(|e| e.to_string())?;
    }

    Ok(())
}
