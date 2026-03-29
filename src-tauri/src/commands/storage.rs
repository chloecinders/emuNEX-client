use std::path::PathBuf;
use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::store;

fn move_dir_contents(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;

    let mut stack: Vec<PathBuf> = vec![src.clone()];
    while let Some(current) = stack.pop() {
        for entry in std::fs::read_dir(&current)
            .map_err(|e| e.to_string())?
            .flatten()
        {
            let path = entry.path();
            let rel = path.strip_prefix(src).map_err(|e| e.to_string())?;
            let dest = dst.join(rel);

            if path.is_dir() {
                std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
                stack.push(path);
            } else {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }

                if std::fs::rename(&path, &dest).is_err() {
                    std::fs::copy(&path, &dest).map_err(|e| e.to_string())?;
                    std::fs::remove_file(&path).ok();
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_data_dir<R: Runtime>(app: AppHandle<R>) -> Result<String, String> {
    let path = store::get_base_dir(&app)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn pick_directory<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    let result = app.dialog().file().blocking_pick_folder();
    Ok(result.and_then(|fp| fp.into_path().ok()).map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
pub fn open_data_dir<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let path = store::get_base_dir(&app)?;
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_custom_data_path<R: Runtime>(
    app: AppHandle<R>,
    new_path: String,
) -> Result<(), String> {
    let new_dir = PathBuf::from(&new_path);
    if !new_dir.is_absolute() {
        return Err("Path must be absolute".to_string());
    }

    let old_dir = store::get_base_dir(&app)?;

    if old_dir == new_dir {
        return Ok(());
    }

    std::fs::create_dir_all(&new_dir).map_err(|e| e.to_string())?;

    if let Ok(entries) = std::fs::read_dir(&old_dir) {
        for entry in entries.flatten() {
            let src = entry.path();
            let name = entry.file_name();
            let dst = new_dir.join(&name);

            if src.is_dir() {
                move_dir_contents(&src, &dst)?;
                std::fs::remove_dir_all(&src).ok();
            } else if src.is_file() {
                let fname = name.to_string_lossy();
                if fname == "store.json" {
                    continue;
                }
                if std::fs::rename(&src, &dst).is_err() {
                    std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
                    std::fs::remove_file(&src).ok();
                }
            }
        }
    }

    let global = store::get_global_store(&app)?;
    global.set("custom_base_path", serde_json::json!(new_path));
    global.save().map_err(|e| e.to_string())?;

    Ok(())
}
