use std::{
    fs::{copy, create_dir_all, remove_file, rename},
    path::PathBuf,
};
use tauri::{AppHandle, Runtime};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use tokio::sync::oneshot;

use crate::store;

fn move_dir_contents(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }

    create_dir_all(dst).map_err(|e| e.to_string())?;

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
                create_dir_all(&dest).map_err(|e| e.to_string())?;
                stack.push(path);
            } else {
                if let Some(parent) = dest.parent() {
                    create_dir_all(parent).map_err(|e| e.to_string())?;
                }

                if rename(&path, &dest).is_err() {
                    copy(&path, &dest).map_err(|e| e.to_string())?;
                    remove_file(&path).ok();
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
pub async fn pick_directory<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    let (tx, rx) = oneshot::channel();

    app.dialog().file().pick_folder(move |result| {
        let path = result
            .and_then(|fp| fp.into_path().ok())
            .map(|p| p.to_string_lossy().into_owned());
        let _ = tx.send(path);
    });

    rx.await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_data_dir<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let path = store::get_base_dir(&app)?;
    if !path.exists() {
        create_dir_all(&path).map_err(|e| e.to_string())?;
    }

    let path_str = path.to_string_lossy().to_string();

    #[cfg(target_os = "linux")]
    {
        let uri = if path_str.starts_with('/') {
            format!("file://{}", path_str)
        } else {
            path_str.clone()
        };

        if std::process::Command::new("xdg-open")
            .arg(&uri)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
    }

    app.opener()
        .open_path(path_str, None::<&str>)
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

    create_dir_all(&new_dir).map_err(|e| e.to_string())?;

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

                if rename(&src, &dst).is_err() {
                    copy(&src, &dst).map_err(|e| e.to_string())?;
                    remove_file(&src).ok();
                }
            }
        }
    }

    let global = store::get_global_store(&app)?;
    global.set("custom_base_path", serde_json::json!(new_path));
    global.save().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn open_external_url<R: Runtime>(app: AppHandle<R>, url: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        if std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
    }

    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}
