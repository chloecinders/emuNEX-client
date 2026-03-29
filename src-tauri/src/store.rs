use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::{Store, StoreExt};

pub fn get_global_store<R: Runtime>(app: &AppHandle<R>) -> Result<Arc<Store<R>>, String> {
    app.store("store.json").map_err(|e| e.to_string())
}

pub fn get_base_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let global = get_global_store(app).ok();
    if let Some(ref store) = global {
        if let Some(custom) = store
            .get("custom_base_path")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
        {
            let p = PathBuf::from(custom);
            if p.is_absolute() {
                return Ok(p);
            }
        }
    }
    app.path().app_data_dir().map_err(|e| e.to_string())
}

pub fn normalize_domain(domain: &str) -> String {
    let mut domain = domain.to_lowercase();

    if !domain.contains("://") {
        let is_local = domain.starts_with("127.0.0.1")
            || domain.starts_with("localhost")
            || domain
                .split(':')
                .next()
                .unwrap_or("")
                .split('.')
                .all(|s| s.parse::<u32>().is_ok());

        let protocol = if is_local { "http://" } else { "https://" };
        domain = format!("{}{}", protocol, domain);
    }

    if let Ok(url) = url::Url::parse(&domain) {
        let mut origin = url.origin().ascii_serialization();

        if url.host_str() == Some("localhost") {
            origin = origin.replace("localhost", "127.0.0.1");
        }

        origin
    } else {
        domain.trim().to_lowercase()
    }
}

pub fn get_domain_folder<R: Runtime>(app: &AppHandle<R>, domain: &str) -> Result<PathBuf, String> {
    let normalized = normalize_domain(domain);
    let mut base = get_base_dir(app)?;
    let safe_domain = normalized
        .replace(|c: char| !c.is_alphanumeric(), "_")
        .to_lowercase();
    base.push("domains");
    base.push(safe_domain);
    Ok(base)
}

pub fn get_domain_store<R: Runtime>(
    app: &AppHandle<R>,
    domain: &str,
) -> Result<Arc<Store<R>>, String> {
    let normalized = normalize_domain(domain);
    let folder = get_domain_folder(app, &normalized)?;
    let _ = std::fs::create_dir_all(&folder);
    let store_path = folder.join("store.json");
    app.store(store_path).map_err(|e| e.to_string())
}

pub fn get_current_domain<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    let store = get_global_store(app).ok()?;
    store
        .get("domain")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

pub fn get_current_store<R: Runtime>(app: &AppHandle<R>) -> Result<Arc<Store<R>>, String> {
    if let Some(domain) = get_current_domain(app) {
        let store = get_domain_store(app, &domain)?;
        if store.get("domain").is_none() {
            let _ = store.set("domain", serde_json::json!(domain));
            let _ = store.save();
        }
        Ok(store)
    } else {
        get_global_store(app)
    }
}

pub fn get_data_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    if let Some(domain) = get_current_domain(app) {
        get_domain_folder(app, &domain)
    } else {
        get_base_dir(app)
    }
}
