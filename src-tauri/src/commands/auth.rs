use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ApiResponse {
    success: bool,
    data: Option<LoginData>,
}

#[derive(Deserialize, Serialize)]
pub struct LoginData {
    login_url: String,
    storage_path: String,
}

#[tauri::command]
pub async fn get_client_start(server_url: String) -> Result<LoginData, String> {
    let mut url = reqwest::Url::parse(&server_url).map_err(|_| "Invalid server URL")?;

    url.set_path("/api/v1/start");

    let res: ApiResponse = reqwest::get(url)
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|_| "Failed to parse server response")?;

    if res.success {
        res.data
            .map(|d| d)
            .ok_or_else(|| "Server success but missing data".to_string())
    } else {
        Err("Server emulation start failed".to_string())
    }
}
