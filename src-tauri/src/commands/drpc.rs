use discord_rich_presence::{
    activity::{self, ActivityType, StatusDisplayType},
    DiscordIpc, DiscordIpcClient,
};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use std::time::Duration;
use tauri::async_runtime::Mutex;
use thiserror::Error;
use tokio::{task::JoinHandle, time::sleep};

#[derive(Error, Debug, Serialize)]
pub enum DrpcError {
    #[error("Discord RPC thread already spawned")]
    ThreadAlreadySpawned,
    #[error("Discord RPC thread not found")]
    ThreadNotFound,
    #[error("Discord IPC error: {0}")]
    IpcError(String),
}

impl From<DrpcError> for String {
    fn from(err: DrpcError) -> Self {
        err.to_string()
    }
}

static RPC_THREAD: LazyLock<Mutex<Option<JoinHandle<()>>>> = LazyLock::new(Mutex::default);
static ACTIVITY: LazyLock<Mutex<Option<DrpcActivity>>> = LazyLock::new(Mutex::default);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrpcTimestamps {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrpcAssets {
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrpcButton {
    pub label: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DrpcActivity {
    pub state: Option<String>,
    pub details: Option<String>,
    pub timestamps: Option<DrpcTimestamps>,
    pub assets: Option<DrpcAssets>,
    pub buttons: Option<Vec<DrpcButton>>,
    pub activity_type: Option<i32>,
    pub status_display_type: Option<i32>,
}

#[tauri::command]
pub async fn spawn_drpc_thread(id: String) -> Result<(), DrpcError> {
    let mut guard = RPC_THREAD.lock().await;

    if guard.is_some() {
        return Err(DrpcError::ThreadAlreadySpawned);
    }

    *guard = Some(tokio::spawn(async move {
        if let Err(e) = initialize_rpc(id).await {
            eprintln!("Discord RPC error: {}", e);
        }
    }));

    Ok(())
}

#[tauri::command]
pub async fn destroy_drpc_thread() -> Result<(), DrpcError> {
    let mut guard = RPC_THREAD.lock().await;

    if let Some(thread) = guard.as_ref() {
        thread.abort();
        *guard = None;
        return Ok(());
    }

    Err(DrpcError::ThreadNotFound)
}

#[tauri::command]
pub async fn is_drpc_running() -> bool {
    let guard = RPC_THREAD.lock().await;
    guard.as_ref().map(|t| !t.is_finished()).unwrap_or(false)
}

#[tauri::command]
pub async fn set_drpc_activity(activity: DrpcActivity) -> Result<(), String> {
    *ACTIVITY.lock().await = Some(activity);
    Ok(())
}

#[tauri::command]
pub async fn clear_drpc_activity() -> Result<(), String> {
    *ACTIVITY.lock().await = None;
    Ok(())
}

async fn get_current_activity() -> Option<DrpcActivity> {
    ACTIVITY.lock().await.as_ref().cloned()
}

async fn initialize_rpc(id: String) -> Result<(), DrpcError> {
    let mut client = DiscordIpcClient::new(&id);

    loop {
        if client.connect().is_ok() {
            break;
        }

        sleep(Duration::from_secs(3)).await;
    }

    loop {
        if let Some(drpc_act) = get_current_activity().await {
            let mut payload = activity::Activity::new();

            if let Some(state) = &drpc_act.state {
                payload = payload.state(state);
            }

            if let Some(details) = &drpc_act.details {
                payload = payload.details(details);
            }

            if let Some(kind) = drpc_act.activity_type {
                let activity_kind = match kind {
                    2 => ActivityType::Listening,
                    3 => ActivityType::Watching,
                    5 => ActivityType::Competing,
                    _ => ActivityType::Playing,
                };

                payload = payload.activity_type(activity_kind);
            }

            if let Some(display) = drpc_act.status_display_type {
                let status_kind = match display {
                    1 => StatusDisplayType::State,
                    2 => StatusDisplayType::Details,
                    _ => StatusDisplayType::Name,
                };

                payload = payload.status_display_type(status_kind);
            }

            if let Some(a) = &drpc_act.assets {
                let mut assets = activity::Assets::new();

                if let Some(li) = &a.large_image {
                    assets = assets.large_image(li);
                }

                if let Some(lt) = &a.large_text {
                    assets = assets.large_text(lt);
                }

                if let Some(si) = &a.small_image {
                    assets = assets.small_image(si);
                }

                if let Some(st) = &a.small_text {
                    assets = assets.small_text(st);
                }

                payload = payload.assets(assets);
            }

            if let Some(t) = &drpc_act.timestamps {
                let mut timestamps = activity::Timestamps::new();

                if let Some(s) = t.start {
                    timestamps = timestamps.start(s);
                }

                if let Some(e) = t.end {
                    timestamps = timestamps.end(e);
                }

                payload = payload.timestamps(timestamps);
            }

            if let Some(btns) = &drpc_act.buttons {
                let payload_btns: Vec<activity::Button> = btns
                    .iter()
                    .map(|b| activity::Button::new(&b.label, &b.url))
                    .collect();

                payload = payload.buttons(payload_btns);
            }

            if client.set_activity(payload).is_err() {
                let _ = client.reconnect();
            }
        }

        sleep(Duration::from_secs(1)).await;
    }
}
