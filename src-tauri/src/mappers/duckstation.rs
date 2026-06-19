use std::collections::HashMap;

fn emunex_to_duckstation(emunex_mapping: &str, api_prefix: &str) -> Option<String> {
    if let Some(code) = emunex_mapping.strip_prefix("Kbd.") {
        let mapped = match code {
            "Enter" => "Return",
            "Escape" => "Escape",
            "Backspace" => "Backspace",
            "Tab" => "Tab",
            "Space" => "Space",
            "ArrowLeft" => "Left",
            "ArrowUp" => "Up",
            "ArrowRight" => "Right",
            "ArrowDown" => "Down",
            "ShiftLeft" | "ShiftRight" => "Shift",
            "ControlLeft" | "ControlRight" => "Control",
            "AltLeft" | "AltRight" => "Alt",
            "PageUp" => "Prior",
            "PageDown" => "Next",
            "Home" => "Home",
            "End" => "End",
            "Insert" => "Insert",
            "Delete" => "Delete",
            "NumpadAdd" => "Keypad+",
            "NumpadSubtract" => "Keypad-",
            c if c.starts_with("Key") => &c[3..],
            c if c.starts_with("Digit") => &c[5..],
            c if c.starts_with("F") => c,
            _ => return None,
        };
        return Some(format!("Keyboard/{}", mapped));
    }

    if let Some(rest) = emunex_mapping.strip_prefix("Joy") {
        let dot = rest.find('.')?;
        let joy_idx: u32 = rest[..dot].parse().unwrap_or(0);
        let control = &rest[dot + 1..];

        if let Some(btn_str) = control.strip_prefix("Btn") {
            let btn_idx: u32 = btn_str.parse().ok()?;
            let btn_name = match btn_idx {
                0 => "A",
                1 => "B",
                2 => "X",
                3 => "Y",
                4 => "LeftShoulder",
                5 => "RightShoulder",
                6 => "+LeftTrigger",
                7 => "+RightTrigger",
                8 => "Back",
                9 => "Start",
                10 => "LeftStick",
                11 => "RightStick",
                12 => "DPadUp",
                13 => "DPadDown",
                14 => "DPadLeft",
                15 => "DPadRight",
                _ => return None,
            };
            return Some(format!("{}-{}/{}", api_prefix, joy_idx, btn_name));
        }

        if let Some(axis_str) = control.strip_prefix("Axis") {
            let (idx_str, dir) = axis_str.split_at(axis_str.len() - 1);
            let axis_idx: u32 = idx_str.parse().ok()?;
            let axis_name = match axis_idx {
                0 => "LeftX",
                1 => "LeftY",
                2 => "RightX",
                3 => "RightY",
                4 => "LeftTrigger",
                5 => "RightTrigger",
                _ => return None,
            };
            return Some(format!("{}-{}/{}{}", api_prefix, joy_idx, dir, axis_name));
        }
    }
    None
}

fn get_duckstation_api_prefix(ini_content: &str) -> String {
    let mut in_input_sources = false;
    for line in ini_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_input_sources = trimmed == "[InputSources]";
            continue;
        }
        if in_input_sources {
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim();
                if val == "true" {
                    if key == "SDL" {
                        return "SDL".to_string();
                    } else if key == "XInput" {
                        return "XInput".to_string();
                    } else if key == "DInputSource" {
                        return "DInput".to_string();
                    } else if key == "RawInput" {
                        return "RawInput".to_string();
                    }
                }
            }
        }
    }

    let mut in_pad1 = false;
    for line in ini_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_pad1 = trimmed == "[Pad1]";
            continue;
        }
        if in_pad1 {
            if let Some((_, val)) = line.split_once('=') {
                let val = val.trim();
                if let Some(slash) = val.find('/') {
                    let full_prefix = &val[..slash];
                    if let Some(dash) = full_prefix.rfind('-') {
                        let api = &full_prefix[..dash];
                        if api != "Keyboard" {
                            return api.to_string();
                        }
                    }
                }
            }
        }
    }
    "SDL".to_string()
}

pub fn map_duckstation(ini_content: &str, global_mappings: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(ini_content.len());
    let mut in_pad1 = false;

    let api_prefix = get_duckstation_api_prefix(ini_content);

    let target_keys = std::collections::HashMap::from([
        ("Cross", "A"),
        ("Circle", "B"),
        ("Square", "X"),
        ("Triangle", "Y"),
        ("L1", "L"),
        ("R1", "R"),
        ("L2", "ZL"),
        ("R2", "ZR"),
        ("L3", "LS Click"),
        ("R3", "RS Click"),
        ("Select", "Select"),
        ("Start", "Start"),
        ("Up", "DPad-Up"),
        ("Down", "DPad-Down"),
        ("Left", "DPad-Left"),
        ("Right", "DPad-Right"),
        ("LUp", "LS Up"),
        ("LDown", "LS Down"),
        ("LLeft", "LS Left"),
        ("LRight", "LS Right"),
        ("RUp", "RS Up"),
        ("RDown", "RS Down"),
        ("RLeft", "RS Left"),
        ("RRight", "RS Right"),
    ]);

    for line in ini_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if trimmed == "[Pad1]" {
                in_pad1 = true;
            } else {
                in_pad1 = false;
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if in_pad1 {
            if let Some(eq_idx) = line.find('=') {
                let key = line[..eq_idx].trim();
                if let Some(global_key) = target_keys.get(key) {
                    if let Some(mapping) = global_mappings.get(*global_key) {
                        if let Some(duckstation_mapping) =
                            emunex_to_duckstation(mapping, &api_prefix)
                        {
                            out.push_str(&format!("{} = {}\n", key, duckstation_mapping));
                            continue;
                        }
                    }
                }
            }
        }

        out.push_str(line);
        out.push('\n');
    }

    out
}
