use futures_util::StreamExt;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::Ordering;
use std::{collections::HashMap, process::Command};
use tauri::{AppHandle, Emitter, Runtime};

use crate::commands::emulator::StoreEmulator;
use crate::commands::save::upload_save_files;
use crate::{store, AppState};

fn split_args(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for c in cmd.chars() {
        if escaped {
            current.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            in_quotes = !in_quotes;
        } else if c.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                args.push(current.clone());
                current.clear();
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// Metadata used for snapshot diffing.
struct FileMeta {
    size: u64,
    modified: Option<std::time::SystemTime>,
}

/// Recursively snapshot all files under `dir`, mapping relative paths to their metadata.
fn recursive_snapshot(dir: &std::path::Path) -> HashMap<std::path::PathBuf, FileMeta> {
    let mut map = HashMap::new();
    if !dir.exists() {
        return map;
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    let meta = entry.metadata().ok();
                    let file_meta = FileMeta {
                        size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
                        modified: meta.as_ref().and_then(|m| m.modified().ok()),
                    };
                    let rel = path.strip_prefix(dir).unwrap_or(&path).to_path_buf();
                    map.insert(rel, file_meta);
                }
            }
        }
    }
    map
}

/// Recursively copy all files from `src_dir` into `dst_dir`, preserving sub-folder structure.
fn recursive_copy_dir(src_dir: &std::path::Path, dst_dir: &std::path::Path) {
    if !src_dir.exists() {
        return;
    }
    let mut stack = vec![src_dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let rel = match path.strip_prefix(src_dir) {
                    Ok(r) => r.to_path_buf(),
                    Err(_) => continue,
                };
                let dest = dst_dir.join(&rel);
                if path.is_dir() {
                    std::fs::create_dir_all(&dest).ok();
                    stack.push(path);
                } else if path.is_file() {
                    if let Some(parent) = dest.parent() {
                        std::fs::create_dir_all(parent).ok();
                    }
                    std::fs::copy(&path, &dest).ok();
                }
            }
        }
    }
}

/// Collect files matching `extensions` under `dir`, returning (relative, absolute) pairs.
fn collect_by_extension(
    dir: &std::path::Path,
    extensions: &[String],
) -> Vec<(std::path::PathBuf, std::path::PathBuf)> {
    let mut results = Vec::new();
    if !dir.exists() || extensions.is_empty() {
        return results;
    }
    let exts_lower: Vec<String> = extensions
        .iter()
        .map(|e| e.trim_start_matches('.').to_lowercase())
        .collect();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    let file_ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.to_lowercase())
                        .unwrap_or_default();
                    if exts_lower.contains(&file_ext) {
                        let rel = path.strip_prefix(dir).unwrap_or(&path).to_path_buf();
                        results.push((rel, path));
                    }
                }
            }
        }
    }
    results
}

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

    let global_store = store::get_global_store(&app)?;
    let stored_emulators: HashMap<String, StoreEmulator> = global_store
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

    let all_emulators: Vec<&StoreEmulator> = stored_emulators
        .values()
        .collect();

    if all_emulators.is_empty() {
        state.is_game_running.store(false, Ordering::SeqCst);
        return Err("Emulator not found for this console".to_string());
    }

    let emulator = if let Some(id) = emulator_id {
        all_emulators
            .into_iter()
            .find(|e| e.id == id)
            .cloned()
            .ok_or_else(|| {
                state.is_game_running.store(false, Ordering::SeqCst);
                "Requested emulator not found".to_string()
            })?
    } else {
        all_emulators
            .iter()
            .find(|e| e.is_default)
            .copied()
            .or_else(|| all_emulators.first().copied())
            .cloned()
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

    let rom_name = persistent_rom_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let data_dir = base_path.to_string_lossy().to_string();
    let save_dir_str = save_dir.to_string_lossy().to_string();
    let temp_dir_str = temp_dir.to_string_lossy().to_string();
    let emu_dir_str = emu_dir.to_string_lossy().to_string();
    let bin_name = std::path::Path::new(&emulator.binary_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_string();

    let rom_path_escaped = format!("'{}'", temp_rom_path.to_string_lossy().replace("'", "''"));

    let resolved_save_path = if let Some(ref template) = emulator.save_path {
        if template.is_empty() {
            None
        } else {
            let path = template
                .replace("$rom_name", rom_name)
                .replace("$game_id", &game_id)
                .replace("$console", &console)
                .replace("$data_dir", &data_dir)
                .replace("$save_dir", &save_dir_str)
                .replace("$temp_dir", &temp_dir_str)
                .replace("$emu_dir", &emu_dir_str);
            Some(path)
        }
    } else {
        None
    };

    // --- Restore saves into sandbox (preserving sub-folder structure) ---
    // If there's an absolute external save_path, restore there; otherwise restore into temp_dir.
    if let Some(ref path) = resolved_save_path {
        let p = std::path::Path::new(path);
        if p.is_absolute() {
            if p.is_file() || p.extension().is_some() {
                // save_path points to a single save file: find a matching file in save_dir
                if let Some(parent) = p.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                let target_name = p.file_name().and_then(|f| f.to_str()).unwrap_or("");
                if !target_name.is_empty() {
                    let src = save_dir.join(target_name);
                    if src.exists() {
                        std::fs::copy(&src, p).ok();
                    }
                }
            } else {
                // save_path points to a directory: recursively restore everything there
                std::fs::create_dir_all(p).ok();
                recursive_copy_dir(&save_dir, p);
            }
        }
    } else {
        // No external save_path: restore into the sandbox temp_dir
        recursive_copy_dir(&save_dir, &temp_dir);
    }



    // --- Pre-launch snapshot (used as diff fallback if no save_extensions configured) ---
    let scan_dir_for_snapshot = if let Some(ref path) = resolved_save_path {
        let p = std::path::Path::new(path);
        if p.is_absolute() && p.is_dir() {
            p.to_path_buf()
        } else {
            temp_dir.clone()
        }
    } else {
        temp_dir.clone()
    };
    let pre_snapshot = recursive_snapshot(&scan_dir_for_snapshot);

    let child = if !emulator.binary_path.is_empty() || !emulator.run_command.is_empty() {
        let bin_to_use = if !emulator.binary_path.is_empty() {
            &emulator.binary_path
        } else {
            &bin_name
        };

        let cmd_template = if !emulator.run_command.is_empty() {
            let cmd = emulator.run_command.clone();
            if !cmd.contains("$bin") && !cmd.contains("$exe") && !emulator.binary_path.is_empty() {
                format!("$exe {}", cmd)
            } else {
                cmd
            }
        } else {
            format!("$exe $rom")
        };

        let parts = split_args(&cmd_template);
        let mut final_parts = Vec::new();

        for part in parts {
            let replaced = part
                .replace("$bin", bin_to_use)
                .replace("$exe", bin_to_use)
                .replace("$rom", &temp_rom_path.to_string_lossy())
                .replace("$rom_name", rom_name)
                .replace("$game_id", &game_id)
                .replace("$console", &console)
                .replace("$data_dir", &data_dir)
                .replace("$save_dir", &save_dir_str)
                .replace("$temp_dir", &temp_dir_str)
                .replace("$emu_dir", &emu_dir_str);
            final_parts.push(replaced);
        }

        if final_parts.is_empty() {
            state.is_game_running.store(false, Ordering::SeqCst);
            return Err("Resolved command is empty".to_string());
        }

        let program = &final_parts[0];
        let args = &final_parts[1..];

        Command::new(program)
            .args(args)
            .current_dir(&temp_dir)
            .spawn()
    } else {
        state.is_game_running.store(false, Ordering::SeqCst);
        return Err("No binary or run command configured".to_string());
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

    // --- Post-launch save detection ---
    // Determine which directory to scan for save files.
    let scan_dir = if let Some(ref path) = resolved_save_path {
        let p = std::path::Path::new(path);
        if p.is_absolute() && (p.is_dir() || (!p.exists() && p.extension().is_none())) {
            p.to_path_buf()
        } else {
            temp_dir.clone()
        }
    } else {
        temp_dir.clone()
    };

    // ROM filename to exclude
    let rom_abs = temp_rom_path.clone();

    if !emulator.save_extensions.is_empty() {
        // PRIMARY: extension matching — collect all files with matching extensions
        let matched = collect_by_extension(&scan_dir, &emulator.save_extensions);
        for (rel, abs_path) in matched {
            let dest = save_dir.join(&rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::copy(&abs_path, &dest).ok();
        }
    } else {
        // FALLBACK: snapshot diffing — copy new or modified files (excl. ROM and config files)
        let post_snapshot = recursive_snapshot(&scan_dir);
        for (rel, post_meta) in &post_snapshot {
            let abs_path = scan_dir.join(rel);
            // Skip the ROM itself
            if abs_path == rom_abs {
                continue;
            }
            // Skip known config files
            let fname = rel
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(""
            );
            if emulator.config_files.iter().any(|c| c == fname) {
                continue;
            }
            let is_new_or_changed = match pre_snapshot.get(rel) {
                None => true, // new file
                Some(pre_meta) => {
                    pre_meta.size != post_meta.size
                        || pre_meta.modified != post_meta.modified
                }
            };
            if is_new_or_changed {
                let dest = save_dir.join(rel);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::copy(&abs_path, &dest).ok();
            }
        }
    }

    // --- Upload to cloud ---
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
