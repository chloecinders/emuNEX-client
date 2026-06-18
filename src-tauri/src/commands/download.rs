use std::collections::{HashMap, VecDeque};
use std::fs::{create_dir_all, remove_file, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime};
use tokio::sync::Notify;

use crate::commands::emulator::{ApiEmulator, StoreEmulator};
use crate::commands::http::{perform_backend_request, BackendBody};
use crate::commands::play::{get_or_create_manifest, InstalledRom};
use crate::{store, ApiResponse};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Done,
    Error(String),
    Cancelled,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DownloadKind {
    Rom {
        game_id: String,
        console: String,
        rom_path: String,
        extension: String,
        name: Option<String>,
        zipped: Option<bool>,
        zipped_entry: Option<String>,
        md5: Option<String>,
    },
    Emulator {
        console: String,
        emulator_id: Option<String>,
        keep_config: Option<bool>,
        source_server: Option<String>,
    },
}

#[derive(Serialize, Clone, Debug)]
pub struct DownloadTask {
    pub id: String,
    pub label: String,
    pub kind: DownloadKind,
    pub status: DownloadStatus,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub speed_bps: u64,
    pub queued_at: u64,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    #[serde(skip)]
    pub cancel_flag: Arc<AtomicBool>,
}

pub struct DownloadManager {
    pub queue: Mutex<VecDeque<DownloadTask>>,
    pub notify: Arc<Notify>,
}

impl DownloadManager {
    pub fn new() -> (Self, Arc<Notify>) {
        let notify = Arc::new(Notify::new());
        let mgr = Self {
            queue: Mutex::new(VecDeque::new()),
            notify: notify.clone(),
        };
        (mgr, notify)
    }

    pub fn active_count(&self) -> usize {
        let q = self.queue.lock().unwrap();
        q.iter()
            .filter(|t| {
                matches!(
                    t.status,
                    DownloadStatus::Queued | DownloadStatus::Downloading
                )
            })
            .count()
    }

    pub fn snapshot(&self) -> Vec<DownloadTask> {
        self.queue.lock().unwrap().iter().cloned().collect()
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn next_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering as AO};
    static CTR: AtomicU64 = AtomicU64::new(1);
    format!("dl-{}", CTR.fetch_add(1, AO::SeqCst))
}

fn md5_of_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut context = md5::Context::new();
    let mut buffer = [0; 128 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        context.consume(&buffer[..count]);
    }
    Ok(format!("{:x}", context.finalize()))
}

async fn stream_to_file<R: Runtime>(
    app: &AppHandle<R>,
    task_id: &str,
    cancel_flag: &Arc<AtomicBool>,
    url: &str,
    token: &str,
    dest_path: &Path,
    total_bytes_hint: u64,
) -> Result<u64, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Authorization", token)
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let content_length = response.content_length().unwrap_or(total_bytes_hint);
    let mut stream = response.bytes_stream();

    let mut file = File::create(dest_path)
        .map_err(|e| format!("Cannot create {}: {e}", dest_path.display()))?;

    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now();
    let mut speed_window: VecDeque<(Instant, u64)> = VecDeque::new();

    while let Some(chunk_res) = stream.next().await {
        if cancel_flag.load(Ordering::SeqCst) {
            return Err("Cancelled".to_string());
        }

        let chunk = chunk_res.map_err(|e| format!("Stream error: {e}"))?;
        file.write_all(&chunk).map_err(|e| format!("Write error: {e}"))?;
        downloaded += chunk.len() as u64;

        let now = Instant::now();
        speed_window.push_back((now, chunk.len() as u64));
        while speed_window
            .front()
            .map(|(t, _)| now.duration_since(*t).as_secs() >= 3)
            .unwrap_or(false)
        {
            speed_window.pop_front();
        }
        let window_bytes: u64 = speed_window.iter().map(|(_, b)| b).sum();
        let speed_bps = if speed_window.len() > 1 {
            let oldest = speed_window.front().unwrap().0;
            let elapsed_ms = now.duration_since(oldest).as_millis().max(1);
            (window_bytes as u128 * 1000 / elapsed_ms) as u64
        } else {
            0
        };

        if last_emit.elapsed().as_millis() >= 100 {
            let _ = app.emit(
                "download-progress",
                serde_json::json!({
                    "id": task_id,
                    "downloaded": downloaded,
                    "total": content_length,
                    "speed_bps": speed_bps,
                }),
            );
            last_emit = Instant::now();
        }
    }

    let _ = app.emit(
        "download-progress",
        serde_json::json!({
            "id": task_id,
            "downloaded": downloaded,
            "total": content_length,
            "speed_bps": 0u64,
        }),
    );

    Ok(downloaded)
}

async fn run_emulator_download<R: Runtime>(
    app: &AppHandle<R>,
    task: &DownloadTask,
    console: &str,
    emulator_id: &Option<String>,
    keep_config: Option<bool>,
    source_server: &Option<String>,
) -> Result<(), String> {
    let global_store = store::get_global_store(app)?;
    let domain = match source_server {
        Some(d) => d.clone(),
        None => store::get_current_domain(app).ok_or("Domain not found")?,
    };

    let domain_store = store::get_domain_store(app, &domain)?;
    let token = domain_store
        .get("token")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or("Token not found")?;
    let storage_path = domain_store
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

    let api_res = perform_backend_request(
        app,
        reqwest::Method::GET,
        &api_url,
        Some(&token),
        BackendBody::None,
        false,
    )
    .await?
    .into_json::<ApiResponse<Vec<ApiEmulator>>>()?;

    let emulators = api_res.data.ok_or("No emulator data from server")?;

    let emulator = if let Some(id) = emulator_id {
        emulators
            .into_iter()
            .find(|e| &e.id == id)
            .ok_or_else(|| "Emulator not found".to_string())?
    } else {
        emulators
            .into_iter()
            .next()
            .ok_or_else(|| "No emulator available".to_string())?
    };

    let mut app_data_dir = store::get_base_dir(app)?;
    app_data_dir.push("emulators");
    app_data_dir.push(console.to_lowercase());
    let emu_name_safe = emulator.name.replace(' ', "_").to_lowercase();
    app_data_dir.push(&emu_name_safe);
    create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;

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

    stream_to_file(
        app,
        &task.id,
        &task.cancel_flag,
        &download_url,
        &token,
        &local_file_path,
        emulator.file_size as u64,
    )
    .await?;

    if let Some(ref expected_hash) = emulator.md5_hash {
        if !expected_hash.is_empty() {
            let actual = md5_of_file(&local_file_path)?;

            if actual.to_lowercase() != expected_hash.to_lowercase() {
                remove_file(&local_file_path).ok();
                return Err(format!(
                    "MD5 mismatch for emulator binary: expected {expected_hash}, got {actual}"
                ));
            }
        }
    }

    let mut final_binary_path = local_file_path.clone();

    if emulator.zipped {
        let zip_file = File::open(&local_file_path).map_err(|e| e.to_string())?;
        let buffered_reader = std::io::BufReader::with_capacity(128 * 1024, zip_file);
        let mut archive = zip::ZipArchive::new(buffered_reader).map_err(|e| e.to_string())?;

        let mut total_uncompressed: u64 = 0;
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                total_uncompressed += file.size();
            }
        }

        let mut extracted_bytes: u64 = 0;
        let mut last_emit = Instant::now();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file.enclosed_name() {
                Some(path) => app_data_dir.join(path),
                None => continue,
            };
            if (*file.name()).ends_with('/') {
                create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let outfile = File::create(&outpath).map_err(|e| e.to_string())?;
                let mut buffered_writer = std::io::BufWriter::with_capacity(128 * 1024, outfile);
                
                let mut buffer = [0; 128 * 1024];
                loop {
                    let count = std::io::Read::read(&mut file, &mut buffer).map_err(|e| e.to_string())?;
                    if count == 0 {
                        break;
                    }
                    std::io::Write::write_all(&mut buffered_writer, &buffer[..count]).map_err(|e| e.to_string())?;
                    extracted_bytes += count as u64;

                    if last_emit.elapsed().as_millis() >= 100 {
                        let pct = if total_uncompressed > 0 {
                            ((extracted_bytes as f64 / total_uncompressed as f64) * 100.0) as u8
                        } else {
                            0
                        };
                        let _ = app.emit("extraction-progress", serde_json::json!({ "id": task.id, "progress": pct }));
                        last_emit = Instant::now();
                    }
                }
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).ok();
                }
            }
        }

        remove_file(&local_file_path).ok();

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

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&final_binary_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&final_binary_path, perms).ok();
        }
    }

    let data_dir = store::get_base_dir(app)?;
    let save_dir = data_dir.join("saves");
    let temp_dir = std::env::temp_dir();

    let ctx = crate::utils::TemplateContext {
        emu_dir: app_data_dir.to_string_lossy().to_string(),
        data_dir: data_dir.to_string_lossy().to_string(),
        save_dir: save_dir.to_string_lossy().to_string(),
        temp_dir: temp_dir.to_string_lossy().to_string(),
        rom_name: String::new(),
        game_id: String::new(),
        console: console.to_string(),
        rom_path: String::new(),
        bin_path: final_binary_path.to_string_lossy().to_string(),
        documents_dir: crate::utils::get_documents_dir(),
    };

    let current_os = std::env::consts::OS;
    for extra in &emulator.extra_files {
        if task.cancel_flag.load(Ordering::SeqCst) {
            return Err("Cancelled".to_string());
        }

        let install_path_str = match current_os {
            "windows" => &extra.windows_path,
            "linux" => &extra.linux_path,
            "macos" => &extra.macos_path,
            _ => continue,
        };

        if install_path_str.is_empty() || extra.s3_path.is_empty() {
            continue;
        }

        let mut install_path = ctx.resolve_path(install_path_str);
        let s3_filename = extra.s3_path.split('/').last().unwrap_or("").to_string();
        let path_has_no_ext = install_path.extension().is_none()
            && !install_path_str.ends_with('/')
            && !install_path_str.ends_with('\\');

        if path_has_no_ext
            && !s3_filename.is_empty()
            && std::path::Path::new(&s3_filename).extension().is_some()
        {
            install_path = install_path.join(&s3_filename);
        }
        if install_path_str.ends_with('/') || install_path_str.ends_with('\\') {
            if !s3_filename.is_empty() {
                install_path = install_path.join(&s3_filename);
            }
        }

        if let Some(parent) = install_path.parent() {
            create_dir_all(parent).ok();
        }

        let extra_url = format!(
            "{}{}/{}",
            domain.trim_end_matches('/'),
            storage_path,
            extra.s3_path.trim_start_matches('/')
        );

        stream_to_file(
            app,
            &task.id,
            &task.cancel_flag,
            &extra_url,
            &token,
            &install_path,
            0,
        )
        .await
        .map_err(|e| format!("Extra file '{}': {e}", extra.s3_path))?;
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
        if !keep_config.unwrap_or(false) {
            existing.run_command = emulator.run_command.clone();
            existing.save_path = emulator.save_path.clone();
            existing.save_extensions = emulator.save_extensions.clone();
            existing.input_config_file = emulator.input_config_file.clone();
            existing.input_mapper = emulator.input_mapper.clone();
            existing.zipped = emulator.zipped;
            existing.extra_files = emulator.extra_files.clone();
        }
        existing.version = emulator.version.clone();
        existing.source_server = Some(domain.clone());
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
                input_config_file: emulator.input_config_file,
                input_mapper: emulator.input_mapper,
                zipped: emulator.zipped,
                file_size: emulator.file_size,
                version: emulator.version,
                source_server: Some(domain),
                extra_files: emulator.extra_files,
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

async fn run_rom_download<R: Runtime>(
    app: &AppHandle<R>,
    task: &DownloadTask,
    game_id: &str,
    console: &str,
    rom_path: &str,
    extension: &str,
    name: &Option<String>,
    zipped: Option<bool>,
    zipped_entry: &Option<String>,
    expected_md5: &Option<String>,
) -> Result<(), String> {
    let store = store::get_current_store(app)?;
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

    let base_path = store::get_data_dir(app)?;

    let mut rom_dir = base_path.clone();
    rom_dir.push("roms");
    rom_dir.push(console.to_lowercase());
    create_dir_all(&rom_dir).map_err(|e| e.to_string())?;

    let mut save_dir = base_path.clone();
    save_dir.push("saves");
    save_dir.push(game_id);
    create_dir_all(&save_dir).map_err(|e| e.to_string())?;

    let mut companion_filenames = Vec::new();
    let entry_filename = zipped_entry.clone().unwrap_or_else(|| {
        let ext = extension.trim_start_matches('.');
        format!("{}.{}", game_id, ext)
    });

    let is_zipped = zipped.unwrap_or(false);
    let dest_path = if is_zipped {
        rom_dir.join(format!("temp_{}.zip", game_id))
    } else {
        rom_dir.join(&entry_filename)
    };

    let download_url = format!(
        "{}{}/{}",
        domain.trim_end_matches('/'),
        storage_path,
        rom_path.trim_start_matches('/')
    );

    stream_to_file(
        app,
        &task.id,
        &task.cancel_flag,
        &download_url,
        &token,
        &dest_path,
        task.total_bytes,
    )
    .await?;

    if let Some(ref expected) = expected_md5 {
        if !expected.is_empty() {
            let actual = md5_of_file(&dest_path)?;

            if actual.to_lowercase() != expected.to_lowercase() {
                remove_file(&dest_path).ok();
                return Err(format!(
                    "MD5 mismatch: expected {expected}, got {actual}"
                ));
            }
        }
    }

    if is_zipped {
        let zip_file = File::open(&dest_path).map_err(|e| e.to_string())?;
        let buffered_reader = std::io::BufReader::with_capacity(128 * 1024, zip_file);
        let mut archive = zip::ZipArchive::new(buffered_reader).map_err(|e| e.to_string())?;

        let mut total_uncompressed: u64 = 0;
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                total_uncompressed += file.size();
            }
        }

        let mut extracted_bytes: u64 = 0;
        let mut last_emit = Instant::now();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let relative_path = match file.enclosed_name() {
                Some(p) => p.to_path_buf(),
                None => continue,
            };
            let outpath = rom_dir.join(&relative_path);

            if (*file.name()).ends_with('/') {
                create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        create_dir_all(p).map_err(|e| e.to_string())?;
                    }
                }
                let outfile = File::create(&outpath).map_err(|e| e.to_string())?;
                let mut buffered_writer = std::io::BufWriter::with_capacity(128 * 1024, outfile);
                
                let mut buffer = [0; 128 * 1024];
                loop {
                    let count = std::io::Read::read(&mut file, &mut buffer).map_err(|e| e.to_string())?;
                    if count == 0 {
                        break;
                    }
                    std::io::Write::write_all(&mut buffered_writer, &buffer[..count]).map_err(|e| e.to_string())?;
                    extracted_bytes += count as u64;

                    if last_emit.elapsed().as_millis() >= 100 {
                        let pct = if total_uncompressed > 0 {
                            ((extracted_bytes as f64 / total_uncompressed as f64) * 100.0) as u8
                        } else {
                            0
                        };
                        let _ = app.emit("extraction-progress", serde_json::json!({ "id": task.id, "progress": pct }));
                        last_emit = Instant::now();
                    }
                }

                let rel_str = relative_path.to_string_lossy().replace('\\', "/");
                if rel_str != entry_filename {
                    companion_filenames.push(rel_str);
                }
            }
        }

        remove_file(&dest_path).ok();
    }

    let mut manifest = get_or_create_manifest(app).await?;
    manifest.roms.insert(
        game_id.to_string(),
        InstalledRom {
            game_id: game_id.to_string(),
            console: console.to_string(),
            rom_filename: entry_filename,
            _name: name.clone(),
            companion_filenames,
        },
    );
    crate::commands::play::write_manifest_pub(&base_path, &manifest)?;

    Ok(())
}

async fn run_task<R: Runtime>(app: &AppHandle<R>, task: &DownloadTask) -> Result<(), String> {
    match &task.kind {
        DownloadKind::Emulator {
            console,
            emulator_id,
            keep_config,
            source_server,
        } => {
            run_emulator_download(
                app,
                task,
                console,
                emulator_id,
                *keep_config,
                source_server,
            )
            .await
        }
        DownloadKind::Rom {
            game_id,
            console,
            rom_path,
            extension,
            name,
            zipped,
            zipped_entry,
            md5,
        } => {
            run_rom_download(
                app, task, game_id, console, rom_path, extension, name, *zipped, zipped_entry, md5,
            )
            .await
        }
    }
}

pub async fn download_worker<R: Runtime>(
    app: AppHandle<R>,
    manager: Arc<DownloadManager>,
) {
    loop {
        manager.notify.notified().await;

        loop {
            let next_id = {
                let q = manager.queue.lock().unwrap();
                q.iter()
                    .find(|t| t.status == DownloadStatus::Queued)
                    .map(|t| t.id.clone())
            };

            let task_id = match next_id {
                None => break,
                Some(id) => id,
            };

            {
                let mut q = manager.queue.lock().unwrap();
                if let Some(t) = q.iter_mut().find(|t| t.id == task_id) {
                    t.status = DownloadStatus::Downloading;
                    t.started_at = Some(now_ms());
                }
            }

            let task_snapshot = {
                let q = manager.queue.lock().unwrap();
                q.iter().find(|t| t.id == task_id).cloned()
            };

            if let Some(task) = task_snapshot {
                let _ = app.emit(
                    "download-started",
                    serde_json::json!({ "id": &task_id }),
                );

                let result = run_task(&app, &task).await;
                let now = now_ms();
                let cancelled = task.cancel_flag.load(Ordering::SeqCst);

                {
                    let mut q = manager.queue.lock().unwrap();
                    if let Some(t) = q.iter_mut().find(|t| t.id == task_id) {
                        if cancelled {
                            t.status = DownloadStatus::Cancelled;
                        } else {
                            match &result {
                                Ok(()) => {
                                    t.status = DownloadStatus::Done;
                                    t.downloaded_bytes = t.total_bytes;
                                }
                                Err(e) => {
                                    t.status = DownloadStatus::Error(e.clone());
                                }
                            }
                        }
                        t.completed_at = Some(now);
                    }
                }

                match result {
                    Ok(()) if !cancelled => {
                        let _ = app.emit(
                            "download-complete",
                            serde_json::json!({
                                "id": &task_id,
                                "kind": match &task.kind {
                                    DownloadKind::Rom { game_id, .. } => serde_json::json!({ "type": "rom", "game_id": game_id }),
                                    DownloadKind::Emulator { console, .. } => serde_json::json!({ "type": "emulator", "console": console }),
                                }
                            }),
                        );
                    }
                    Err(ref e) if !cancelled => {
                        let _ = app.emit(
                            "download-error",
                            serde_json::json!({ "id": &task_id, "error": e }),
                        );
                    }
                    _ => {
                        let _ = app.emit(
                            "download-error",
                            serde_json::json!({ "id": &task_id, "error": "Cancelled" }),
                        );
                    }
                }

                if manager.active_count() == 0 {
                    if let Some(win) = app.get_webview_window("main") {
                        if !win.is_visible().unwrap_or(true) {
                            app.exit(0);
                        }
                    }

                    if let Some(tray) = app.tray_by_id("main-tray") {
                        let _ = tray.set_visible(false);
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub async fn queue_download_rom<R: Runtime>(
    app: AppHandle<R>,
    manager: tauri::State<'_, Arc<DownloadManager>>,
    label: String,
    game_id: String,
    console: String,
    rom_path: String,
    extension: String,
    name: Option<String>,
    zipped: Option<bool>,
    zipped_entry: Option<String>,
    total_bytes: u64,
    md5: Option<String>,
) -> Result<String, String> {
    let id = next_id();
    let task = DownloadTask {
        id: id.clone(),
        label,
        kind: DownloadKind::Rom {
            game_id,
            console,
            rom_path,
            extension,
            name,
            zipped,
            zipped_entry,
            md5,
        },
        status: DownloadStatus::Queued,
        total_bytes,
        downloaded_bytes: 0,
        speed_bps: 0,
        queued_at: now_ms(),
        started_at: None,
        completed_at: None,
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };

    let _ = app.emit("download-queued", &task);
    manager.queue.lock().unwrap().push_back(task);
    manager.notify.notify_one();

    Ok(id)
}

#[tauri::command]
pub async fn queue_download_emulator<R: Runtime>(
    app: AppHandle<R>,
    manager: tauri::State<'_, Arc<DownloadManager>>,
    label: String,
    console: String,
    emulator_id: Option<String>,
    keep_config: Option<bool>,
    source_server: Option<String>,
    total_bytes: u64,
) -> Result<String, String> {
    let id = next_id();
    let task = DownloadTask {
        id: id.clone(),
        label,
        kind: DownloadKind::Emulator {
            console,
            emulator_id,
            keep_config,
            source_server,
        },
        status: DownloadStatus::Queued,
        total_bytes,
        downloaded_bytes: 0,
        speed_bps: 0,
        queued_at: now_ms(),
        started_at: None,
        completed_at: None,
        cancel_flag: Arc::new(AtomicBool::new(false)),
    };

    let _ = app.emit("download-queued", &task);
    manager.queue.lock().unwrap().push_back(task);
    manager.notify.notify_one();

    Ok(id)
}

#[tauri::command]
pub fn get_download_queue(
    manager: tauri::State<'_, Arc<DownloadManager>>,
) -> Vec<DownloadTask> {
    manager.snapshot()
}

#[tauri::command]
pub fn cancel_download(
    manager: tauri::State<'_, Arc<DownloadManager>>,
    id: String,
) -> Result<(), String> {
    let mut q = manager.queue.lock().unwrap();
    if let Some(task) = q.iter_mut().find(|t| t.id == id) {
        if task.status == DownloadStatus::Downloading {
            task.cancel_flag.store(true, Ordering::SeqCst);
        } else if task.status == DownloadStatus::Queued {
            task.status = DownloadStatus::Cancelled;
        }
        Ok(())
    } else {
        Err("Download not found".to_string())
    }
}

#[tauri::command]
pub fn remove_download(
    manager: tauri::State<'_, Arc<DownloadManager>>,
    id: String,
) -> Result<(), String> {
    let mut q = manager.queue.lock().unwrap();
    q.retain(|t| t.id != id);
    Ok(())
}
