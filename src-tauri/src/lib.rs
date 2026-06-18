use std::{
    fs::{copy, create_dir_all, read_dir, remove_dir_all},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use serde::Deserialize;
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{App, Emitter, Manager, WindowEvent};
#[cfg(desktop)]
use tauri_plugin_deep_link::DeepLinkExt;

mod commands;
mod mappers;
mod store;
mod utils;

#[derive(Deserialize)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
}

pub struct AppState {
    is_game_running: AtomicBool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (download_manager, _notify) = commands::download::DownloadManager::new();
    let download_manager = Arc::new(download_manager);

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            is_game_running: false.into(),
        })
        .manage(download_manager.clone())
        .on_window_event(move |window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();
                let game_running = state.is_game_running.load(Ordering::SeqCst);

                let downloads_active = window
                    .app_handle()
                    .try_state::<Arc<commands::download::DownloadManager>>()
                    .map(|mgr| mgr.active_count() > 0)
                    .unwrap_or(false);

                if game_running {
                    api.prevent_close();
                    let _ = window.emit(
                        "close-prevented",
                        "Game is still running. Please close the emulator first.",
                    );
                } else if downloads_active {
                    api.prevent_close();
                    let _ = window.hide();

                    let count = window
                        .app_handle()
                        .try_state::<Arc<commands::download::DownloadManager>>()
                        .map(|mgr| mgr.active_count())
                        .unwrap_or(0);

                    if let Some(tray) = window.app_handle().tray_by_id("main-tray") {
                        let label = if count == 1 {
                            "emuNEX - 1 download running".to_string()
                        } else {
                            format!("emuNEX - {} downloads running", count)
                        };
                        let _ = tray.set_tooltip(Some(&label));
                        let _ = tray.set_visible(true);
                    }
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
            commands::download::queue_download_rom,
            commands::download::queue_download_emulator,
            commands::download::get_download_queue,
            commands::download::cancel_download,
            commands::download::remove_download,
            commands::emulator::download_emulator,
            commands::emulator::remove_emulator,
            commands::emulator::migrate_emulator_files,
            commands::emulator::fetch_server_emulators,
            commands::emulator::fetch_all_server_emulators,
            commands::emulator::get_emulator_dir_sizes,
            commands::emulator::refresh_emulator_config,
            commands::play::play_game,
            commands::play::install_game,
            commands::play::is_game_installed,
            commands::play::get_local_storage,
            commands::play::delete_installed_rom,
            commands::play::delete_local_save,
            commands::play::update_manifest_names,
            commands::save::check_save_status,
            commands::save::download_save_files,
            commands::save::get_game_save_files,
            commands::save::open_save_folder,
            commands::storage::get_data_dir,
            commands::storage::set_custom_data_path,
            commands::storage::open_data_dir,
            commands::storage::open_external_url,
            commands::storage::pick_directory,
            commands::storage::open_folder,
            commands::drpc::spawn_drpc_thread,
            commands::drpc::destroy_drpc_thread,
            commands::drpc::is_drpc_running,
            commands::drpc::set_drpc_activity,
            commands::drpc::clear_drpc_activity,
            commands::input::apply_scheme_to_emulator,
            commands::input::save_global_inputs,
            commands::input::load_global_inputs,
        ])
        .setup(move |app| startup(app, download_manager.clone()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn startup(
    app: &mut App,
    download_manager: Arc<commands::download::DownloadManager>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    app.deep_link().register("emunex")?;

    let tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("emuNEX - downloads running")
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                if let Some(win) = tray.app_handle().get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                    let _ = tray.set_visible(false);
                }
            }
        })
        .build(app)?;

    tray.set_visible(false)?;

    let handle = app.handle().clone();
    tauri::async_runtime::spawn(commands::download::download_worker(
        handle,
        download_manager,
    ));

    let handle = app.handle().clone();
    let temp_base = std::env::temp_dir();

    if let Ok(entries) = read_dir(temp_base) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("emunex_") {
                let game_id = name.replace("emunex_", "");
                let recovery_path = entry.path();

                let mut save_dir = store::get_data_dir(&handle).unwrap();
                save_dir.push("saves");
                save_dir.push(&game_id);
                let _ = create_dir_all(&save_dir);

                if let Ok(files) = read_dir(&recovery_path) {
                    for file in files.flatten() {
                        let fpath = file.path();
                        let fname = file.file_name().to_string_lossy().into_owned();

                        if fpath.is_file() {
                            if fname.starts_with(&format!("{}.", game_id)) {
                                continue;
                            }

                            let dest = save_dir.join(file.file_name());
                            let _ = copy(&fpath, &dest);
                        }
                    }
                }

                let _ = remove_dir_all(recovery_path);
            }
        }
    }

    Ok(())
}
