use futures_util::StreamExt;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::Ordering;
use std::{collections::HashMap, process::Command};
use tauri::{AppHandle, Manager, Runtime, State};
use tauri_plugin_store::StoreExt;

use crate::commands::emulator::StoreEmulator;
use crate::AppState;

#[tauri::command]
pub async fn install_game<R: Runtime>(
    app: AppHandle<R>,
    game_id: String,
    console: String,
    rom_path: String,
) -> Result<(), String> {
    let store = app.store("store.json").map_err(|e| e.to_string())?;
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

    let base_path = app.path().app_data_dir().map_err(|e| e.to_string())?;

    let mut rom_dir = base_path.clone();
    rom_dir.push("roms");
    rom_dir.push(&console);
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
    state: State<'_, AppState>,
    game_id: String,
    console: String,
) -> Result<(), String> {
    state.is_game_running.store(true, Ordering::SeqCst);

    let store = app.store("store.json").map_err(|e| e.to_string())?;
    let base_path = app.path().app_data_dir().map_err(|e| e.to_string())?;

    let stored_emulators: HashMap<String, StoreEmulator> = store
        .get("emulators")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .ok_or("No emulators")?;
    let emulator = stored_emulators.get(&console).ok_or("Emulator not found")?;

    let mut rom_dir = base_path.clone();
    rom_dir.push("roms");
    rom_dir.push(&console);

    let mut save_dir = base_path.clone();
    save_dir.push("saves");
    save_dir.push(&game_id);
    std::fs::create_dir_all(&save_dir).map_err(|e| e.to_string())?;

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
    let emu_dir = emu_bin_path.parent().unwrap();

    // Copy configs to temp_dir
    for config_name in &emulator.config_files {
        let src = emu_dir.join(config_name);
        if src.exists() {
            std::fs::copy(&src, temp_dir.join(config_name)).ok();
        }
    }

    let temp_rom_path = temp_dir.join(persistent_rom_path.file_name().unwrap());
    std::fs::copy(&persistent_rom_path, &temp_rom_path).map_err(|e| e.to_string())?;

    for entry in std::fs::read_dir(&save_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        std::fs::copy(entry.path(), temp_dir.join(entry.file_name())).ok();
    }

    let mut args: Vec<String> = emulator
        .run_command
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    if !args.is_empty() {
        args.remove(0);
    }
    args.push(temp_rom_path.to_string_lossy().to_string());

    let working_dir = if emulator.zipped { emu_dir } else { &temp_dir };

    let mut child = Command::new(&emulator.binary_path)
        .current_dir(working_dir)
        .args(&args)
        .spawn()
        .map_err(|e| e.to_string())?;

    child.wait().map_err(|e| e.to_string())?;

    for entry in std::fs::read_dir(&temp_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let fname = entry.file_name().to_string_lossy().to_string();

        if path == temp_rom_path || emulator.config_files.contains(&fname) {
            continue;
        }

        let dest = save_dir.join(entry.file_name());
        std::fs::copy(&path, &dest).map_err(|e| e.to_string())?;
    }

    std::fs::remove_dir_all(&temp_dir).ok();
    state.is_game_running.store(false, Ordering::SeqCst);
    Ok(())
}
