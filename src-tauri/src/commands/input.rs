use crate::commands::emulator::StoreEmulator;
use crate::store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tauri::{command, AppHandle, Runtime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupportedEmulator {
    pub id: String,
    pub name: String,
    pub mapper: String,
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
    pub active_scheme_id: Option<String>,
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
        serde_json::to_value(&config).unwrap(),
    );
    let _ = global_store.save();
    Ok("Successfully saved input config globally".into())
}

#[command]
pub async fn load_global_inputs<R: Runtime>(
    app: AppHandle<R>,
) -> Result<serde_json::Value, String> {
    let global_store = store::get_global_store(&app)?;

    let mut config: InputManagerConfig = global_store
        .get("input_manager_config")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_else(|| {
            let mappings: HashMap<String, String> = global_store
                .get("input_mappings")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let layout_mode = global_store
                .get("input_layout_mode")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "xbox".to_string());
            let gamepad_id = global_store
                .get("input_gamepad_id")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            let auto_apply = global_store
                .get("input_auto_apply_on_start")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let default_scheme = InputScheme {
                id: "default-scheme".to_string(),
                name: "Default Profile".to_string(),
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
                        scheme_id: Some("default-scheme".to_string()),
                        auto_apply_on_start: auto_apply,
                    },
                );
            }

            InputManagerConfig {
                schemes: vec![default_scheme],
                active_scheme_id: Some("default-scheme".to_string()),
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
            if !mapper.trim().is_empty() {
                supported.push(SupportedEmulator {
                    id: emu.id.clone(),
                    name: emu.name.clone(),
                    mapper: mapper.clone(),
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

fn map_vbam(config_content: &str, global_mappings: &HashMap<String, String>) -> String {
    let mut out_lines = Vec::new();
    let mut in_target_block = false;

    let convert_value = |v: &str| -> String {
        if v == "Unmapped" || v == "..." || v.is_empty() {
            return "".to_string();
        }
        if v.starts_with("Kbd.") {
            let code = &v[4..];
            return match code {
                "ArrowUp" => "UP".to_string(),
                "ArrowDown" => "DOWN".to_string(),
                "ArrowLeft" => "LEFT".to_string(),
                "ArrowRight" => "RIGHT".to_string(),
                "Enter" => "ENTER".to_string(),
                "Backspace" => "BACK".to_string(),
                "Space" => "SPACE".to_string(),
                "Tab" => "TAB".to_string(),
                "ShiftLeft" | "ShiftRight" => "SHIFT".to_string(),
                "ControlLeft" | "ControlRight" => "CTRL".to_string(),
                "AltLeft" | "AltRight" => "ALT".to_string(),
                c if c.starts_with("Key") => c[3..].to_string(),
                c if c.starts_with("Digit") => c[5..].to_string(),
                _ => "".to_string(),
            };
        }

        if !v.starts_with("Joy") {
            return v.to_string();
        }

        let rest = &v[3..];
        let Some(dot_idx) = rest.find('.') else {
            return v.to_string();
        };

        let joy_idx_str = &rest[..dot_idx];
        let Ok(joy_idx) = joy_idx_str.parse::<u32>() else {
            return v.to_string();
        };

        let joy_1_idx = joy_idx + 1;
        let control = &rest[dot_idx + 1..];

        if control.starts_with("Btn") {
            let mut btn_idx = control[3..].parse::<u32>().unwrap_or(u32::MAX);

            btn_idx = match btn_idx {
                4 => 9,
                5 => 10,
                8 => 4,
                9 => 6,
                12 => 11,
                13 => 12,
                14 => 13,
                15 => 14,
                _ => btn_idx,
            };

            if btn_idx != u32::MAX {
                return format!("Joy{}-Button{}", joy_1_idx, btn_idx);
            } else {
                return v.to_string();
            }
        } else if control.starts_with("Axis") {
            let axis_val = &control[4..];
            return format!("Joy{}-Axis{}", joy_1_idx, axis_val);
        }

        v.to_string()
    };

    let target_keys = std::collections::HashMap::from([
        ("Up", "DPad-Up"),
        ("Down", "DPad-Down"),
        ("Left", "DPad-Left"),
        ("Right", "DPad-Right"),
        ("A", "A"),
        ("B", "B"),
        ("L", "L"),
        ("R", "R"),
        ("Select", "Select"),
        ("Start", "Start"),
    ]);

    for line in config_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_target_block = trimmed == "[Joypad/1]";
            out_lines.push(line.to_string());
            continue;
        }

        if !in_target_block {
            out_lines.push(line.to_string());
            continue;
        }

        let Some(eq_idx) = line.find('=') else {
            out_lines.push(line.to_string());
            continue;
        };

        let key = line[..eq_idx].trim();
        let Some(global_key) = target_keys.get(key) else {
            out_lines.push(line.to_string());
            continue;
        };

        let Some(mapping) = global_mappings.get(*global_key) else {
            out_lines.push(line.to_string());
            continue;
        };

        let new_val = convert_value(mapping);
        out_lines.push(format!("{}={}", key, new_val));
    }

    out_lines.join("\n")
}

fn expand_env_vars(path: &str) -> std::path::PathBuf {
    let mut result = String::new();
    let chars: Vec<char> = path.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '%' {
            let mut j = i + 1;
            let mut found_end = false;
            while j < chars.len() {
                if chars[j] == '%' {
                    found_end = true;
                    break;
                }
                j += 1;
            }
            if found_end {
                let var_name: String = chars[i + 1..j].iter().collect();
                if let Ok(val) = std::env::var(&var_name) {
                    result.push_str(&val);
                } else {
                    let mut matched = false;
                    for (k, v) in std::env::vars() {
                        if k.eq_ignore_ascii_case(&var_name) {
                            result.push_str(&v);
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        result.push_str(&format!("%{}%", var_name));
                    }
                }
                i = j + 1;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    std::path::PathBuf::from(result)
}

pub fn apply_inputs_to_emulator(emu: &StoreEmulator, mappings: &HashMap<String, String>) {
    let Some(mapper) = &emu.input_mapper else {
        return;
    };
    let Some(config_file) = &emu.input_config_file else {
        return;
    };

    let expanded_path = expand_env_vars(config_file);

    let config_path = if expanded_path.is_absolute() {
        expanded_path
    } else {
        let Some(parent) = std::path::Path::new(&emu.binary_path).parent() else {
            return;
        };
        parent.join(expanded_path)
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
