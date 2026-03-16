use std::sync::atomic::{AtomicBool, Ordering};

use serde::Deserialize;
use tauri::{Emitter, Manager, WindowEvent};
#[cfg(desktop)]
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_store::StoreExt;

use crate::commands::emulator::StoreEmulator;

mod commands;
mod store;

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

pub struct AppState {
    is_game_running: AtomicBool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            is_game_running: false.into(),
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                if state.is_game_running.load(Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = window.emit(
                        "close-prevented",
                        "Game is still running. Please close the emulator first.",
                    );
                }
            }
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::auth::get_client_start,
            commands::http::http,
            commands::emulator::download_emulator,
            commands::emulator::fetch_server_emulators,
            commands::emulator::fetch_all_server_emulators,
            commands::play::play_game,
            commands::play::install_game,
            commands::play::is_game_installed,
            commands::play::get_local_storage,
            commands::play::delete_installed_rom,
            commands::play::delete_local_save,
            commands::save::check_save_status,
            commands::save::download_save_files,
        ])
        .setup(|app| {
            #[cfg(desktop)]
            app.deep_link().register("emunex")?;

            let handle = app.handle().clone();
            let temp_base = std::env::temp_dir();

            let store = if let Some(domain) = store::get_current_domain(&handle) {
                store::get_domain_store(&handle, &domain).ok()
            } else {
                handle.store("store.json").ok()
            };

            if let Ok(entries) = std::fs::read_dir(temp_base) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.starts_with("emunex_") {
                        let game_id = name.replace("emunex_", "");
                        let recovery_path = entry.path();

                        let mut save_dir = store::get_data_dir(&handle).unwrap();
                        save_dir.push("saves");
                        save_dir.push(&game_id);
                        let _ = std::fs::create_dir_all(&save_dir);

                        if let Ok(files) = std::fs::read_dir(&recovery_path) {
                            for file in files.flatten() {
                                let fpath = file.path();
                                let fname = file.file_name().to_string_lossy().into_owned();

                                if fpath.is_file() {
                                    if fname.starts_with(&format!("{}.", game_id)) {
                                        continue;
                                    }

                                    let is_config = if let Some(ref s) = store {
                                        s.get("emulators")
                                            .and_then(|v| {
                                                serde_json::from_value::<
                                                    std::collections::HashMap<
                                                        String,
                                                        Vec<StoreEmulator>,
                                                    >,
                                                >(
                                                    v.clone()
                                                )
                                                .ok()
                                            })
                                            .map(|ems| {
                                                ems.values()
                                                    .flatten()
                                                    .any(|e| e.config_files.contains(&fname))
                                            })
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    };

                                    if is_config {
                                        continue;
                                    }

                                    let dest = save_dir.join(file.file_name());
                                    let _ = std::fs::copy(&fpath, &dest);
                                }
                            }
                        }

                        let _ = std::fs::remove_dir_all(recovery_path);
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
