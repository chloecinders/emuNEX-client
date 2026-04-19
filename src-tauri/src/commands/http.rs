use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyRequest {
    url: String,
    method: String,
    body: Option<serde_json::Value>,
    token: Option<String>,
}

#[derive(Serialize)]
pub struct ProxyResponse {
    pub id: u64,
    pub status: u16,
    pub data: serde_json::Value,
}

#[tauri::command]
pub async fn http(req: ProxyRequest) -> Result<ProxyResponse, String> {
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
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        "TRACE" => reqwest::Method::TRACE,
        "CONNECT" => reqwest::Method::CONNECT,
        _ => return Err("Unsupported method".into()),
    };

    let mut request_builder = client.request(method, &req.url).headers(headers);

    if let Some(body) = req.body {
        request_builder = request_builder.json(&body);
    }

    let response = request_builder.send().await.map_err(|e| e.to_string())?;

    let status = response.status();
    let body_bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let data = if let Ok(json) = serde_json::from_slice(&body_bytes) {
        json
    } else {
        serde_json::json!(String::from_utf8_lossy(&body_bytes).to_string())
    };

    let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::SeqCst);

    if !status.is_success() {
        let err_obj = serde_json::json!({
            "id": request_id,
            "status": status.as_u16(),
            "data": data
        });
        return Err(err_obj.to_string());
    }

    Ok(ProxyResponse {
        id: request_id,
        status: status.as_u16(),
        data,
    })
}

use std::time::Instant;
use tauri::{AppHandle, Emitter, Runtime};

pub enum BackendBody {
    Json(serde_json::Value),
    // Multipart(reqwest::multipart::Form),
    None,
}

pub enum BackendResponse {
    Json(serde_json::Value),
    Text(String),
    Stream(reqwest::Response),
}

impl BackendResponse {
    pub fn into_json<T: serde::de::DeserializeOwned>(self) -> Result<T, String> {
        match self {
            BackendResponse::Json(j) => serde_json::from_value(j).map_err(|e| e.to_string()),
            BackendResponse::Text(t) => serde_json::from_str(&t).map_err(|e| e.to_string()),
            _ => Err("Invalid response type: expected JSON".into()),
        }
    }

    pub fn into_stream(self) -> Result<reqwest::Response, String> {
        match self {
            BackendResponse::Stream(s) => Ok(s),
            _ => Err("Invalid response type: expected Stream".into()),
        }
    }
}

pub async fn perform_backend_request<R: Runtime>(
    app: &AppHandle<R>,
    method: reqwest::Method,
    url: &str,
    token: Option<&str>,
    body: BackendBody,
    stream_response: bool,
) -> Result<BackendResponse, String> {
    let start_time = Instant::now();
    let client = reqwest::Client::new();
    let mut builder = client.request(method.clone(), url);

    if let Some(tok) = token {
        builder = builder.header(reqwest::header::AUTHORIZATION, tok);
    }

    let mut log_body = serde_json::Value::Null;
    builder = match body {
        BackendBody::Json(json) => {
            log_body = json.clone();
            builder.json(&json)
        }
        // BackendBody::Multipart(form) => {
        //     log_body = serde_json::json!("<Multipart Form Data>");
        //     builder.multipart(form)
        // }
        BackendBody::None => builder,
    };

    let response = builder.send().await.map_err(|e| e.to_string())?;

    let status = response.status();
    let is_success = status.is_success();
    let log_response;

    let result = if stream_response && is_success {
        log_response = serde_json::json!("<Streaming Response>");
        Ok(BackendResponse::Stream(response))
    } else {
        let bytes = response.bytes().await.map_err(|e| e.to_string())?;
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            log_response = json.clone();
            if is_success {
                Ok(BackendResponse::Json(json))
            } else {
                Err(log_response.to_string())
            }
        } else {
            let txt = String::from_utf8_lossy(&bytes).to_string();
            log_response = serde_json::json!(txt.clone());
            if is_success {
                Ok(BackendResponse::Text(txt))
            } else {
                Err(txt)
            }
        }
    };

    let duration = start_time.elapsed().as_millis() as u64;
    let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::SeqCst);

    let log_event = serde_json::json!({
        "id": request_id.to_string(),
        "method": method.as_str(),
        "url": url,
        "status": status.as_u16(),
        "body": log_body,
        "response": log_response,
        "success": is_success,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        "duration": duration,
    });

    // Emit on a dedicated channel so the main window's DevStore routes this
    // through addRequest(), which re-broadcasts via the frontend emit path and
    // ensures the Request Viewer window receives it reliably.
    let _ = app.emit("dev://backend-request", log_event);

    result
}
