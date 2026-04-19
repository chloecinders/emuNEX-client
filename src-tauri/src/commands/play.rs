use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs::{copy, create_dir_all, read_dir, write, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::{collections::HashMap, process::Command};
use tauri::{AppHandle, Emitter, Runtime};

use crate::commands::emulator::StoreEmulator;
use crate::commands::http::{perform_backend_request, BackendBody};
use crate::commands::save::upload_save_files;
use crate::{store, ApiResponse, AppState};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstalledRom {
    pub game_id: String,
    pub console: String,
    pub rom_filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _name: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct InstalledRomsManifest {
    pub roms: HashMap<String, InstalledRom>,
}

fn manifest_path(data_dir: &Path) -> PathBuf {
    data_dir.join("installed_roms.json")
}

pub fn read_manifest(data_dir: &Path) -> InstalledRomsManifest {
    let path = manifest_path(data_dir);
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(manifest) = serde_json::from_str(&content) {
                return manifest;
            }
        }
    }

    InstalledRomsManifest::default()
}

fn write_manifest(data_dir: &Path, manifest: &InstalledRomsManifest) -> Result<(), String> {
    let path = manifest_path(data_dir);

    if let Some(parent) = path.parent() {
        let _ = create_dir_all(parent);
    }

    let content = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    write(&path, content)
        .map_err(|e| format!("Failed writing manifest to {}: {}", path.display(), e))
}

async fn migrate_to_manifest<R: Runtime>(
    app: &AppHandle<R>,
    data_dir: &Path,
) -> InstalledRomsManifest {
    let mut manifest = InstalledRomsManifest::default();
    let roms_dir = data_dir.join("roms");

    let mut found_game_ids: HashMap<String, (String, String)> = HashMap::new();

    if roms_dir.exists() {
        if let Ok(consoles) = read_dir(&roms_dir) {
            for console_entry in consoles.filter_map(|e| e.ok()) {
                let console_path = console_entry.path();

                if !console_path.is_dir() {
                    continue;
                }

                let fallback_console = console_entry.file_name().to_string_lossy().to_string();
                let files = match read_dir(&console_path) {
                    Ok(f) => f,
                    Err(_) => continue,
                };

                for file_entry in files.filter_map(|e| e.ok()) {
                    let file_name = file_entry.file_name().to_string_lossy().to_string();
                    if let Some(game_id) = file_name.split('.').next() {
                        found_game_ids
                            .insert(game_id.to_string(), (file_name, fallback_console.clone()));
                    }
                }
            }
        }
    }

    if found_game_ids.is_empty() {
        return manifest;
    }

    let ids_to_request: Vec<String> = found_game_ids.keys().cloned().collect();

    #[derive(Serialize)]
    struct BulkRequest {
        ids: Vec<String>,
    }

    #[derive(Deserialize)]
    struct BulkGame {
        id: String,
        console: String,
        #[serde(rename = "title")]
        name: String,
    }

    let mut verified_metadata: HashMap<String, BulkGame> = HashMap::new();

    if let Ok(store) = store::get_current_store(app) {
        let domain_opt = store
            .get("domain")
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        let token_opt = store
            .get("token")
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        if let (Some(domain), Some(token)) = (domain_opt, token_opt) {
            let base_url = format!("{}/api/v1/roms/bulk_info", domain.trim_end_matches('/'));

            for chunk in ids_to_request.chunks(50) {
                let req_body = BulkRequest {
                    ids: chunk.to_vec(),
                };

                let req_payload = BackendBody::Json(
                    serde_json::to_value(&req_body).unwrap_or(serde_json::Value::Null),
                );

                if let Ok(resp) = perform_backend_request(
                    app,
                    reqwest::Method::POST,
                    &base_url,
                    Some(&token),
                    req_payload,
                    false,
                )
                .await
                {
                    if let Ok(api_res) = resp.into_json::<ApiResponse<Vec<BulkGame>>>() {
                        if let Some(data) = api_res.data {
                            for game in data {
                                verified_metadata.insert(game.id.clone(), game);
                            }
                        }
                    }
                }
            }
        }
    }

    for (game_id, (file_name, fallback_console)) in found_game_ids {
        let (console, name) = if let Some(meta) = verified_metadata.get(&game_id) {
            (meta.console.clone(), Some(meta.name.clone()))
        } else {
            (fallback_console, None)
        };

        manifest.roms.insert(
            game_id.clone(),
            InstalledRom {
                game_id,
                console,
                rom_filename: file_name,
                _name: name,
            },
        );
    }

    if let Err(e) = write_manifest(data_dir, &manifest) {
        println!("Error writing manifest: {}", e);
    }

    manifest
}

async fn get_or_create_manifest<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<InstalledRomsManifest, String> {
    let data_dir = store::get_data_dir(app)?;
    let path = manifest_path(&data_dir);
    if path.exists() {
        Ok(read_manifest(&data_dir))
    } else {
        Ok(migrate_to_manifest(app, &data_dir).await)
    }
}

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

struct FileMeta {
    size: u64,
    modified: Option<std::time::SystemTime>,
}

fn recursive_snapshot(dir: &Path) -> HashMap<PathBuf, FileMeta> {
    let mut map = HashMap::new();

    if !dir.exists() {
        return map;
    }

    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match read_dir(&current) {
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
    map
}

fn recursive_copy_dir(src_dir: &Path, dst_dir: &Path) {
    if !src_dir.exists() {
        return;
    }

    let mut stack = vec![src_dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let rel = match path.strip_prefix(src_dir) {
                Ok(r) => r.to_path_buf(),
                Err(_) => continue,
            };

            let dest = dst_dir.join(&rel);

            if path.is_dir() {
                create_dir_all(&dest).ok();
                stack.push(path);
            } else if path.is_file() {
                if let Some(parent) = dest.parent() {
                    create_dir_all(parent).ok();
                }

                copy(&path, &dest).ok();
            }
        }
    }
}

fn collect_by_extension(dir: &Path, extensions: &[String]) -> Vec<(PathBuf, PathBuf)> {
    let mut results = Vec::new();

    if !dir.exists() || extensions.is_empty() {
        return results;
    }

    let exts_lower: Vec<String> = extensions
        .iter()
        .map(|e| {
            let lower = e.to_lowercase();
            if lower.starts_with('.') {
                lower
            } else {
                format!(".{}", lower)
            }
        })
        .collect();

    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let path_str = path.to_string_lossy().to_lowercase();

                if exts_lower.iter().any(|ext| path_str.ends_with(ext)) {
                    let rel = path.strip_prefix(dir).unwrap_or(&path).to_path_buf();
                    results.push((rel, path));
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
    name: Option<String>,
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
    rom_dir.push(&console.to_lowercase());
    create_dir_all(&rom_dir).map_err(|e| e.to_string())?;

    let mut save_dir = base_path.clone();
    save_dir.push("saves");
    save_dir.push(&game_id);
    create_dir_all(&save_dir).map_err(|e| e.to_string())?;

    let extension = Path::new(&rom_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("rom");
    let file_name = format!("{}.{}", game_id, extension);
    rom_dir.push(&file_name);

    if !rom_dir.exists() {
        let download_url = format!(
            "{}{}/{}",
            domain.trim_end_matches('/'),
            storage_path,
            rom_path.trim_start_matches('/')
        );

        let response = perform_backend_request(
            &app,
            reqwest::Method::GET,
            &download_url,
            Some(&token),
            BackendBody::None,
            true,
        )
        .await?
        .into_stream()?;

        let mut stream = response.bytes_stream();
        let mut file = File::create(&rom_dir).map_err(|e| e.to_string())?;

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| e.to_string())?;
            file.write_all(&chunk).map_err(|e| e.to_string())?;
        }
    }

    let mut manifest = get_or_create_manifest(&app).await?;
    manifest.roms.insert(
        game_id.clone(),
        InstalledRom {
            game_id,
            console,
            rom_filename: file_name,
            _name: name,
        },
    );
    write_manifest(&base_path, &manifest)?;

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
    create_dir_all(&save_dir).map_err(|e| e.to_string())?;

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
        .filter(|e| e.consoles.iter().any(|c| c == &console))
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

    if let Some(config) = global_store.get("input_manager_config").and_then(|v| {
        serde_json::from_value::<crate::commands::input::InputManagerConfig>(v.clone()).ok()
    }) {
        if let Some(emu_cfg) = config.emulator_configs.get(&emulator.id) {
            if emu_cfg.auto_apply_on_start {
                if let Some(scheme) = emu_cfg
                    .scheme_id
                    .as_ref()
                    .and_then(|id| config.schemes.iter().find(|s| &s.id == id))
                {
                    crate::commands::input::apply_inputs_to_emulator(&emulator, &scheme.mappings);
                }
            }
        }
    }

    let manifest = get_or_create_manifest(&app).await?;
    let rom_info = manifest
        .roms
        .get(&game_id)
        .ok_or("ROM not found in manifest")?;

    let persistent_rom_path = base_path
        .join("roms")
        .join(&rom_info.console.to_lowercase())
        .join(&rom_info.rom_filename);

    if !persistent_rom_path.exists() {
        return {
            state.is_game_running.store(false, Ordering::SeqCst);
            Err("ROM file missing from disk".to_string())
        };
    }

    let temp_dir = std::env::temp_dir().join(format!("emunex_{}", game_id));
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir).ok();
    }
    create_dir_all(&temp_dir).map_err(|e| e.to_string())?;

    let emu_bin_path = Path::new(&emulator.binary_path);
    let emu_dir = emu_bin_path.parent().unwrap_or_else(|| Path::new(""));

    let rom_filename = persistent_rom_path.file_name().ok_or("Invalid ROM path")?;
    let temp_rom_path = temp_dir.join(rom_filename);
    copy(&persistent_rom_path, &temp_rom_path).map_err(|e| e.to_string())?;

    for entry in read_dir(&save_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        copy(entry.path(), temp_dir.join(entry.file_name())).ok();
    }

    let rom_name = persistent_rom_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let data_dir = base_path.to_string_lossy().to_string();
    let save_dir_str = save_dir.to_string_lossy().to_string();
    let temp_dir_str = temp_dir.to_string_lossy().to_string();
    let emu_dir_str = emu_dir.to_string_lossy().to_string();
    let bin_name = Path::new(&emulator.binary_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_string();

    let _rom_path_escaped = format!("'{}'", temp_rom_path.to_string_lossy().replace("'", "''"));

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

    if let Some(ref path) = resolved_save_path {
        let p = Path::new(path);
        if p.is_absolute() {
            if p.is_dir() && p.extension().is_none() {
                create_dir_all(p).ok();
                recursive_copy_dir(&save_dir, p);
            } else {
                if let Some(parent) = p.parent() {
                    create_dir_all(parent).ok();
                }
                if let Some(target_name) = p.file_name().and_then(|f| f.to_str()) {
                    if !target_name.is_empty() {
                        let src = save_dir.join(target_name);
                        if src.exists() {
                            copy(&src, p).ok();
                        }
                    }
                }
            }
        }
    } else {
        recursive_copy_dir(&save_dir, &temp_dir);
    }

    let scan_dir_for_snapshot = if let Some(ref path) = resolved_save_path {
        let p = Path::new(path);
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

    let scan_dir = if let Some(ref path) = resolved_save_path {
        let p = Path::new(path);

        if p.is_absolute() && (p.is_dir() || (!p.exists() && p.extension().is_none())) {
            p.to_path_buf()
        } else {
            temp_dir.clone()
        }
    } else {
        temp_dir.clone()
    };

    let rom_abs = temp_rom_path.clone();

    if !emulator.save_extensions.is_empty() {
        let matched = collect_by_extension(&scan_dir, &emulator.save_extensions);
        for (rel, abs_path) in matched {
            let dest = save_dir.join(&rel);

            if let Some(parent) = dest.parent() {
                create_dir_all(parent).ok();
            }

            if copy(&abs_path, &dest).is_ok() && scan_dir != temp_dir {
                let _ = std::fs::remove_file(&abs_path);
            }
        }
    } else {
        let post_snapshot = recursive_snapshot(&scan_dir);

        for (rel, post_meta) in &post_snapshot {
            let abs_path = scan_dir.join(rel);

            if abs_path == rom_abs {
                continue;
            }

            let is_new_or_changed = match pre_snapshot.get(rel) {
                None => true,
                Some(pre_meta) => {
                    pre_meta.size != post_meta.size || pre_meta.modified != post_meta.modified
                }
            };

            if is_new_or_changed {
                let dest = save_dir.join(rel);

                if let Some(parent) = dest.parent() {
                    create_dir_all(parent).ok();
                }

                if copy(&abs_path, &dest).is_ok() && scan_dir != temp_dir {
                    let _ = std::fs::remove_file(&abs_path);
                }
            } else if scan_dir != temp_dir {
                let _ = std::fs::remove_file(&abs_path);
            }
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
pub async fn is_game_installed<R: Runtime>(
    app: AppHandle<R>,
    game_id: String,
) -> Result<bool, String> {
    let base_path = store::get_data_dir(&app)?;
    let manifest = get_or_create_manifest(&app).await?;

    if let Some(rom_info) = manifest.roms.get(&game_id) {
        let rom_path = base_path
            .join("roms")
            .join(&rom_info.console.to_lowercase())
            .join(&rom_info.rom_filename);
        Ok(rom_path.exists())
    } else {
        Ok(false)
    }
}

#[derive(serde::Serialize, Clone)]
pub struct LocalStorageEntry {
    pub console: Option<String>,
    pub game_id: String,
    pub rom_size: u64,
    pub save_size: u64,
    pub local_version: Option<i32>,
}

fn get_dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }

    let mut total = 0;

    if let Ok(entries) = read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            total += get_dir_size(&entry.path());
        }
    }

    total
}

#[tauri::command]
pub async fn get_local_storage<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Vec<LocalStorageEntry>, String> {
    let base_path = store::get_data_dir(&app)?;

    let global_store = match store::get_current_store(&app) {
        Ok(s) => Some(s),
        Err(_) => None,
    };

    let sync_versions = global_store
        .as_ref()
        .and_then(|s| s.get("sync_versions"))
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    let manifest = get_or_create_manifest(&app).await?;
    let mut entries: HashMap<String, LocalStorageEntry> = HashMap::new();

    for (game_id, rom_info) in &manifest.roms {
        let rom_path = base_path
            .join("roms")
            .join(&rom_info.console)
            .join(&rom_info.rom_filename);
        let rom_size = if rom_path.exists() {
            rom_path.metadata().map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let local_version = sync_versions
            .get(game_id)
            .and_then(|v| v.as_i64().map(|i| i as i32));
        entries.insert(
            game_id.clone(),
            LocalStorageEntry {
                game_id: game_id.clone(),
                console: Some(rom_info.console.clone()),
                rom_size,
                save_size: 0,
                local_version,
            },
        );
    }

    let saves_dir = base_path.join("saves");

    if saves_dir.exists() {
        if let Ok(games) = read_dir(&saves_dir) {
            for game_entry in games.filter_map(|e| e.ok()) {
                let game_ref = game_entry.file_name().to_string_lossy().to_string();
                let size = get_dir_size(&game_entry.path());

                if size > 0 {
                    let local_version = sync_versions
                        .get(&game_ref)
                        .and_then(|v| v.as_i64().map(|i| i as i32));

                    entries
                        .entry(game_ref.clone())
                        .and_modify(|e| {
                            e.save_size = size;
                            if e.local_version.is_none() {
                                e.local_version = local_version;
                            }
                        })
                        .or_insert(LocalStorageEntry {
                            game_id: game_ref,
                            console: None,
                            rom_size: 0,
                            save_size: size,
                            local_version,
                        });
                }
            }
        }
    }

    Ok(entries.into_values().collect())
}

#[tauri::command]
pub async fn delete_installed_rom<R: Runtime>(
    app: AppHandle<R>,
    game_id: String,
) -> Result<(), String> {
    let base_path = store::get_data_dir(&app)?;
    let mut manifest = get_or_create_manifest(&app).await?;

    if let Some(rom_info) = manifest.roms.remove(&game_id) {
        let rom_path = base_path
            .join("roms")
            .join(&rom_info.console.to_lowercase())
            .join(&rom_info.rom_filename);
        if rom_path.exists() {
            std::fs::remove_file(&rom_path).map_err(|e| e.to_string())?;
        }
        write_manifest(&base_path, &manifest)?;
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

#[tauri::command]
pub async fn update_manifest_names<R: Runtime>(
    app: AppHandle<R>,
    updates: HashMap<String, String>,
) -> Result<(), String> {
    let base_path = store::get_data_dir(&app)?;
    let mut manifest = get_or_create_manifest(&app).await?;
    let mut changed = false;

    for (id, name) in updates {
        if let Some(rom) = manifest.roms.get_mut(&id) {
            if rom._name.as_ref() != Some(&name) {
                rom._name = Some(name);
                changed = true;
            }
        }
    }

    if changed {
        let _ = write_manifest(&base_path, &manifest);
    }
    Ok(())
}
