use crate::commands::emulator::StoreEmulator;
use crate::mappers::{
    duckstation::map_duckstation, mesen::map_mesen, mgba::map_mgba, project64::map_project64,
    snes9x::map_snes9x, vbam::map_vbam,
};
use crate::utils::TemplateContext;
use crate::{store, utils};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::{command, AppHandle, Runtime};

const SUPPORTED_MAPPERS: [&str; 6] = [
    "vbam",
    "mesen",
    "mgba",
    "snes9x",
    "project64",
    "duckstation",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupportedEmulator {
    pub id: String,
    pub name: String,
    pub input_mapper: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InputScheme {
    pub id: String,
    pub name: String,
    pub mappings: HashMap<String, String>,
    pub layout_mode: String,
    pub gamepad_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EmulatorInputConfig {
    pub scheme_id: Option<String>,
    pub auto_apply_on_start: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InputManagerConfig {
    pub schemes: Vec<InputScheme>,
    pub emulator_configs: HashMap<String, EmulatorInputConfig>,
}

#[command]
pub async fn save_global_inputs<R: Runtime>(
    app: AppHandle<R>,
    config: InputManagerConfig,
) -> Result<String, String> {
    let global_store = store::get_global_store(&app)?;
    global_store.set(
        "input_manager_config",
        serde_json::to_value(&config).map_err(|e| e.to_string())?,
    );
    global_store.save().map_err(|e| e.to_string())?;
    Ok("Successfully saved input config globally".into())
}

#[command]
pub async fn load_global_inputs<R: Runtime>(
    app: AppHandle<R>,
) -> Result<serde_json::Value, String> {
    let global_store = store::get_global_store(&app)?;

    let mut config: InputManagerConfig = global_store
        .get("input_manager_config")
        .and_then(|v| {
            let res: Result<InputManagerConfig, _> = serde_json::from_value(v.clone());
            if let Err(ref e) = res {
                eprintln!("Failed to deserialize input_manager_config: {}", e);
            }
            res.ok()
        })
        .unwrap_or_else(|| {
            let mappings: HashMap<String, String> = global_store
                .get("input_mappings")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_else(|| {
                    let mut m = HashMap::new();
                    m.insert("DPad-Up".to_string(), "Joy0.Btn12".to_string());
                    m.insert("DPad-Down".to_string(), "Joy0.Btn13".to_string());
                    m.insert("DPad-Left".to_string(), "Joy0.Btn14".to_string());
                    m.insert("DPad-Right".to_string(), "Joy0.Btn15".to_string());
                    m.insert("LS Up".to_string(), "Joy0.Axis1-".to_string());
                    m.insert("LS Down".to_string(), "Joy0.Axis1+".to_string());
                    m.insert("LS Left".to_string(), "Joy0.Axis0-".to_string());
                    m.insert("LS Right".to_string(), "Joy0.Axis0+".to_string());
                    m.insert("RS Up".to_string(), "Joy0.Axis3-".to_string());
                    m.insert("RS Down".to_string(), "Joy0.Axis3+".to_string());
                    m.insert("A".to_string(), "Joy0.Btn0".to_string());
                    m.insert("B".to_string(), "Joy0.Btn1".to_string());
                    m.insert("X".to_string(), "Joy0.Btn2".to_string());
                    m.insert("Y".to_string(), "Joy0.Btn3".to_string());
                    m.insert("L".to_string(), "Joy0.Btn4".to_string());
                    m.insert("R".to_string(), "Joy0.Btn5".to_string());
                    m.insert("ZL".to_string(), "Joy0.Btn6".to_string());
                    m.insert("ZR".to_string(), "Joy0.Btn7".to_string());
                    m.insert("Select".to_string(), "Joy0.Btn8".to_string());
                    m.insert("Start".to_string(), "Joy0.Btn9".to_string());
                    m.insert("LS Click".to_string(), "Joy0.Btn10".to_string());
                    m.insert("RS Click".to_string(), "Joy0.Btn11".to_string());
                    m.insert("confirm".to_string(), "Kbd.Enter".to_string());
                    m.insert("back".to_string(), "Kbd.Backspace".to_string());
                    m.insert("up".to_string(), "Kbd.ArrowUp".to_string());
                    m.insert("down".to_string(), "Kbd.ArrowDown".to_string());
                    m.insert("left".to_string(), "Kbd.ArrowLeft".to_string());
                    m.insert("right".to_string(), "Kbd.ArrowRight".to_string());
                    m
                });
            let layout_mode = global_store
                .get("input_layout_mode")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "xbox".to_string());
            let gamepad_id = global_store
                .get("input_gamepad_id")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "any".to_string());
            let auto_apply = global_store
                .get("input_auto_apply_on_start")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let default_scheme = InputScheme {
                id: "emunex_default".to_string(),
                name: "emuNEX (Built-in)".to_string(),
                mappings,
                layout_mode,
                gamepad_id,
            };

            let stored_emulators: HashMap<String, StoreEmulator> = global_store
                .get("emulators")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let mut emu_configs = HashMap::new();

            for (id, _) in stored_emulators {
                emu_configs.insert(
                    id.clone(),
                    EmulatorInputConfig {
                        scheme_id: Some("emunex_default".to_string()),
                        auto_apply_on_start: auto_apply,
                    },
                );
            }

            InputManagerConfig {
                schemes: vec![default_scheme],
                emulator_configs: emu_configs,
            }
        });

    let stored_emulators: HashMap<String, StoreEmulator> = global_store
        .get("emulators")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let mut supported = Vec::new();

    for (_, emu) in &stored_emulators {
        if let Some(mapper) = &emu.input_mapper {
            if SUPPORTED_MAPPERS.contains(&mapper.as_str()) {
                supported.push(SupportedEmulator {
                    id: emu.id.clone(),
                    name: emu.name.clone(),
                    input_mapper: mapper.clone(),
                });

                if !config.emulator_configs.contains_key(&emu.id) {
                    config.emulator_configs.insert(
                        emu.id.clone(),
                        EmulatorInputConfig {
                            scheme_id: None,
                            auto_apply_on_start: false,
                        },
                    );
                }
            }
        }
    }

    let mut val = serde_json::to_value(config).unwrap();
    val.as_object_mut().unwrap().insert(
        "supported_emulators".to_string(),
        serde_json::to_value(supported).unwrap(),
    );

    Ok(val)
}

pub fn apply_inputs_to_emulator(emu: &StoreEmulator, mappings: &HashMap<String, String>) {
    let Some(mapper) = &emu.input_mapper else {
        return;
    };
    let Some(config_file) = &emu.input_config_file else {
        return;
    };

    let ctx = TemplateContext {
        emu_dir: Path::new(&emu.binary_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        documents_dir: utils::get_documents_dir(),
        ..Default::default()
    };

    let config_path = ctx.resolve_path(config_file);

    let config_path = if config_path.is_absolute() {
        config_path
    } else {
        let Some(parent) = Path::new(&emu.binary_path).parent() else {
            return;
        };
        parent.join(config_path)
    };

    if !config_path.exists() {
        return;
    }

    let Ok(content) = fs::read_to_string(&config_path) else {
        return;
    };

    let m_low = mapper.to_lowercase();
    match m_low.as_str() {
        "vbam" => {
            let new_content = map_vbam(&content, mappings);
            let _ = fs::write(&config_path, new_content);
        }
        "mesen" => {
            let new_content = map_mesen(&content, mappings);
            let _ = fs::write(&config_path, new_content);
        }
        "mgba" => {
            let new_content = map_mgba(&content, mappings);
            let _ = fs::write(&config_path, new_content);
        }
        "snes9x" => {
            let new_content = map_snes9x(&content, mappings);
            let _ = fs::write(&config_path, new_content);
        }
        "project64" => {
            let new_content = map_project64(&content, mappings);
            let _ = fs::write(&config_path, new_content);
        }
        "duckstation" => {
            let new_content = map_duckstation(&content, mappings);
            let _ = fs::write(&config_path, new_content);
        }
        _ => {}
    }
}

#[command]
pub async fn apply_scheme_to_emulator<R: Runtime>(
    app: AppHandle<R>,
    emulator_id: String,
    scheme_id: String,
) -> Result<bool, String> {
    let global_store = store::get_global_store(&app)?;

    let stored_emulators: HashMap<String, StoreEmulator> = global_store
        .get("emulators")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let emu = stored_emulators
        .get(&emulator_id)
        .ok_or("Emulator not found")?;

    let config: crate::commands::input::InputManagerConfig = global_store
        .get("input_manager_config")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let scheme = config
        .schemes
        .iter()
        .find(|s| s.id == scheme_id)
        .ok_or("Scheme not found")?;

    apply_inputs_to_emulator(emu, &scheme.mappings);

    Ok(true)
}
