use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyRequest {
    url: String,
    method: String,
    body: Option<serde_json::Value>,
    token: Option<String>,
}

#[tauri::command]
pub async fn http(req: ProxyRequest) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    if let Some(token) = req.token {
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&token).map_err(|e| e.to_string())?,
        );
    }

    let method = match req.method.as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        _ => return Err("Unsupported method".into()),
    };

    let mut request_builder = client.request(method, &req.url).headers(headers);

    if let Some(body) = req.body {
        request_builder = request_builder.json(&body);
    }

    let response = request_builder.send().await.map_err(|e| e.to_string())?;

    let status = response.status();
    let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(data.to_string());
    }

    Ok(data)
}
