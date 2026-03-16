use futures_util::StreamExt;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::Ordering;
use std::{collections::HashMap, process::Command};
use tauri::{AppHandle, Emitter, Runtime};

use crate::commands::emulator::StoreEmulator;
use crate::commands::save::upload_save_files;
use crate::{store, AppState};

#[tauri::command]
pub async fn install_game<R: Runtime>(
    app: AppHandle<R>,
    game_id: String,
    console: String,
    rom_path: String,
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
    let storage_path = store
        .get("storage_path")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("No storage_path")?;

    let base_path = store::get_data_dir(&app)?;

    let mut rom_dir = base_path.clone();
    rom_dir.push("roms");
    let console_lowercased = console.to_lowercase();
    rom_dir.push(&console_lowercased);
    std::fs::create_dir_all(&rom_dir).map_err(|e| e.to_string())?;

    let mut save_dir = base_path.clone();
    save_dir.push("saves");
    save_dir.push(&game_id);
    std::fs::create_dir_all(&save_dir).map_err(|e| e.to_string())?;

    let extension = std::path::Path::new(&rom_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("rom");
    let file_name = format!("{}.{}", game_id, extension);
    rom_dir.push(file_name);

    if !rom_dir.exists() {
        let client = reqwest::Client::new();
        let download_url = format!(
            "{}{}/{}",
            domain.trim_end_matches('/'),
            storage_path,
            rom_path.trim_start_matches('/')
        );

        let response = client
            .get(&download_url)
            .header("Authorization", &token)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let mut stream = response.bytes_stream();
        let mut file = File::create(&rom_dir).map_err(|e| e.to_string())?;

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| e.to_string())?;
            file.write_all(&chunk).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn play_game<R: Runtime>(
    app: AppHandle<R>,
    state: tauri::State<'_, AppState>,
    game_id: String,
    console: String,
    emulator_id: Option<String>,
) -> Result<(), String> {
    let store = store::get_current_store(&app)?;
    let base_path = store::get_data_dir(&app)?;

    let mut save_dir = base_path.clone();
    save_dir.push("saves");
    save_dir.push(&game_id);
    std::fs::create_dir_all(&save_dir).map_err(|e| e.to_string())?;

    if state.is_game_running.load(Ordering::SeqCst) {
        return Err("A game is already running".to_string());
    }

    state.is_game_running.store(true, Ordering::SeqCst);

    let stored_emulators: HashMap<String, Vec<StoreEmulator>> = store
        .get("emulators")
        .and_then(|v| {
            serde_json::from_value(v.clone())
                .map_err(|e| {
                    println!("Failed to deserialize emulators: {}", e);
                    e
                })
                .ok()
        })
        .unwrap_or_default();

    if stored_emulators.is_empty() {
        state.is_game_running.store(false, Ordering::SeqCst);
        return Err(
            "No emulators configured. Please add an emulator in the management page.".to_string(),
        );
    }

    let console_emulators = stored_emulators
        .get(&console)
        .or_else(|| {
            let lower = console.to_lowercase();
            stored_emulators
                .iter()
                .find(|(k, _)| k.to_lowercase() == lower)
                .map(|(_, v)| v)
        })
        .ok_or_else(|| {
            state.is_game_running.store(false, Ordering::SeqCst);
            "Emulator not found for this console".to_string()
        })?;

    let emulator = if let Some(id) = emulator_id {
        console_emulators
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| {
                state.is_game_running.store(false, Ordering::SeqCst);
                "Requested emulator not found".to_string()
            })?
    } else {
        console_emulators
            .iter()
            .find(|e| e.is_default)
            .or_else(|| console_emulators.first())
            .ok_or_else(|| {
                state.is_game_running.store(false, Ordering::SeqCst);
                "No default emulator found".to_string()
            })?
    };

    let mut rom_dir = base_path.clone();
    rom_dir.push("roms");
    let console_lowercased = console.to_lowercase();
    rom_dir.push(&console_lowercased);

    let persistent_rom_path = std::fs::read_dir(&rom_dir)
        .map_err(|_| "ROM directory missing")?
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{}.", game_id))
        })
        .map(|e| e.path())
        .ok_or("ROM not found")?;

    let temp_dir = std::env::temp_dir().join(format!("emunex_{}", game_id));
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).ok();
    }
    std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let emu_bin_path = std::path::Path::new(&emulator.binary_path);
    let emu_dir = emu_bin_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new(""));

    for config_name in &emulator.config_files {
        let src = emu_dir.join(config_name);
        if src.exists() {
            std::fs::copy(&src, temp_dir.join(config_name)).ok();
        }
    }

    let rom_filename = persistent_rom_path.file_name().ok_or("Invalid ROM path")?;
    let temp_rom_path = temp_dir.join(rom_filename);
    std::fs::copy(&persistent_rom_path, &temp_rom_path).map_err(|e| e.to_string())?;

    for entry in std::fs::read_dir(&save_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        std::fs::copy(entry.path(), temp_dir.join(entry.file_name())).ok();
    }

    let child = if !emulator.binary_path.is_empty() {
        Command::new(&emulator.binary_path)
            .current_dir(emu_dir)
            .arg(&temp_rom_path)
            .spawn()
    } else if !emulator.run_command.is_empty() {
        let rom_path_escaped = format!("'{}'", temp_rom_path.to_string_lossy().replace("'", "''"));
        let substituted_cmd = emulator.run_command.replace("$rom", &rom_path_escaped);

        #[cfg(windows)]
        {
            Command::new("powershell")
                .args(&[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &substituted_cmd,
                ])
                .current_dir(&temp_dir)
                .spawn()
        }
        #[cfg(not(windows))]
        {
            Command::new("sh")
                .args(&["-c", &substituted_cmd])
                .current_dir(&temp_dir)
                .spawn()
        }
    } else {
        state.is_game_running.store(false, Ordering::SeqCst);
        return Err("No binary path or run command specified.".to_string());
    };

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            state.is_game_running.store(false, Ordering::SeqCst);
            let _ = app.emit("game-status", "Stopped");
            return Err(format!("Failed to launch: {}", e));
        }
    };

    let _ = app.emit("game-status", "Playing");

    child.wait().map_err(|e| e.to_string())?;

    let rom_name = persistent_rom_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    for entry in std::fs::read_dir(&temp_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();

        if path == temp_rom_path || emulator.config_files.contains(&fname) {
            continue;
        }

        let should_copy = if let Some(ref save_path_template) = emulator.save_path {
            if save_path_template.is_empty() {
                true
            } else {
                let substituted = save_path_template.replace("$rom_name", rom_name);
                let target_path = std::path::Path::new(&substituted);

                if target_path.is_absolute() {
                    fname
                        == target_path
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("")
                } else {
                    let relative_fname = target_path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("");
                    fname == relative_fname
                }
            }
        } else {
            true
        };

        if should_copy {
            std::fs::copy(&path, save_dir.join(entry.file_name())).map_err(|e| e.to_string())?;
        }
    }

    let mut sync_versions = store
        .get("sync_versions")
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    let current_v = sync_versions
        .get(&game_id)
        .and_then(|v| v.as_i64().map(|i| i as i32))
        .unwrap_or(0);

    let new_v = current_v + 1;
    upload_save_files(app.clone(), &game_id, &save_dir, new_v)
        .await
        .ok();

    sync_versions.insert(game_id, serde_json::json!(new_v));
    store.set("sync_versions", serde_json::json!(sync_versions));
    let _ = store.save();

    std::fs::remove_dir_all(&temp_dir).ok();
    state.is_game_running.store(false, Ordering::SeqCst);
    let _ = app.emit("game-status", "Stopped");
    Ok(())
}

#[tauri::command]
pub fn is_game_installed<R: Runtime>(
    app: AppHandle<R>,
    game_id: String,
    console: String,
) -> Result<bool, String> {
    let base_path = store::get_data_dir(&app)?;

    let mut rom_dir = base_path.clone();
    rom_dir.push("roms");
    let console_lowercased = console.to_lowercase();
    rom_dir.push(&console_lowercased);

    if !rom_dir.exists() {
        return Ok(false);
    }

    let exists = std::fs::read_dir(&rom_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{}.", game_id))
        });

    Ok(exists)
}

#[derive(serde::Serialize, Clone)]
pub struct LocalStorageEntry {
    pub console: Option<String>,
    pub game_id: String,
    pub rom_size: u64,
    pub save_size: u64,
}

fn get_dir_size(path: &std::path::Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }

    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            total += get_dir_size(&entry.path());
        }
    }
    total
}

#[tauri::command]
pub fn get_local_storage<R: Runtime>(app: AppHandle<R>) -> Result<Vec<LocalStorageEntry>, String> {
    let base_path = store::get_data_dir(&app)?;

    let mut entries: HashMap<String, LocalStorageEntry> = HashMap::new();

    let mut roms_dir = base_path.clone();
    roms_dir.push("roms");
    if roms_dir.exists() {
        if let Ok(consoles) = std::fs::read_dir(&roms_dir) {
            for console_entry in consoles.filter_map(|e| e.ok()) {
                let console = console_entry.file_name().to_string_lossy().to_string();
                let console_path = console_entry.path();
                if console_path.is_dir() {
                    if let Ok(files) = std::fs::read_dir(&console_path) {
                        for file_entry in files.filter_map(|e| e.ok()) {
                            let file_name = file_entry.file_name().to_string_lossy().to_string();
                            if let Some(game_id) = file_name.split('.').next() {
                                let size = file_entry.metadata().map(|m| m.len()).unwrap_or(0);
                                entries.insert(
                                    game_id.to_string(),
                                    LocalStorageEntry {
                                        game_id: game_id.to_string(),
                                        console: Some(console.clone()),
                                        rom_size: size,
                                        save_size: 0,
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    let mut saves_dir = base_path.clone();
    saves_dir.push("saves");
    if saves_dir.exists() {
        if let Ok(games) = std::fs::read_dir(&saves_dir) {
            for game_entry in games.filter_map(|e| e.ok()) {
                let game_ref = game_entry.file_name().to_string_lossy().to_string();
                let size = get_dir_size(&game_entry.path());
                if size > 0 {
                    entries
                        .entry(game_ref.clone())
                        .and_modify(|e| {
                            e.save_size = size;
                        })
                        .or_insert(LocalStorageEntry {
                            game_id: game_ref,
                            console: None,
                            rom_size: 0,
                            save_size: size,
                        });
                }
            }
        }
    }

    Ok(entries.into_values().collect())
}

#[tauri::command]
pub fn delete_installed_rom<R: Runtime>(
    app: AppHandle<R>,
    game_id: String,
    console: String,
) -> Result<(), String> {
    let base_path = store::get_data_dir(&app)?;

    let mut rom_dir = base_path.clone();
    rom_dir.push("roms");
    let console_lowercased = console.to_lowercase();
    rom_dir.push(&console_lowercased);

    if !rom_dir.exists() {
        return Ok(());
    }

    if let Ok(files) = std::fs::read_dir(&rom_dir) {
        for file_entry in files.filter_map(|e| e.ok()) {
            let file_name = file_entry.file_name().to_string_lossy().to_string();
            if file_name.starts_with(&format!("{}.", game_id)) {
                std::fs::remove_file(file_entry.path()).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn delete_local_save<R: Runtime>(app: AppHandle<R>, game_id: String) -> Result<(), String> {
    let base_path = store::get_data_dir(&app)?;

    let save_dir = base_path.join("saves").join(&game_id);

    if save_dir.exists() {
        std::fs::remove_dir_all(save_dir).map_err(|e| e.to_string())?;
    }

    Ok(())
}
