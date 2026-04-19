use std::{
    fs::{copy, create_dir_all, read_dir, remove_dir_all},
    sync::atomic::{AtomicBool, Ordering},
};

use serde::Deserialize;
use tauri::{App, Emitter, Manager, WindowEvent};
#[cfg(desktop)]
use tauri_plugin_deep_link::DeepLinkExt;

mod commands;
mod store;

#[derive(Deserialize)]
struct ApiResponse<T> {
    // success: bool,
    data: Option<T>,
    // message: Option<String>,
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
            commands::emulator::remove_emulator,
            commands::emulator::migrate_emulator_files,
            commands::emulator::fetch_server_emulators,
            commands::emulator::fetch_all_server_emulators,
            commands::emulator::get_emulator_dir_sizes,
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
            commands::storage::pick_directory,
            commands::drpc::spawn_drpc_thread,
            commands::drpc::destroy_drpc_thread,
            commands::drpc::is_drpc_running,
            commands::drpc::set_drpc_activity,
            commands::drpc::clear_drpc_activity,
            commands::input::apply_scheme_to_emulator,
            commands::input::save_global_inputs,
            commands::input::load_global_inputs,
        ])
        .setup(startup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn startup(app: &mut App) -> std::result::Result<(), Box<dyn std::error::Error>> {
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    app.deep_link().register("emunex")?;

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
