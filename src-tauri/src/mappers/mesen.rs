use std::collections::HashMap;

fn emunex_to_mesen_keycode(v: &str) -> Option<u32> {
    if v.is_empty() || v == "Unmapped" || v == "..." {
        return Some(0);
    }

    if let Some(code) = v.strip_prefix("Kbd.") {
        let vk: u32 = match code {
            "KeyA" => 0x41,
            "KeyB" => 0x42,
            "KeyC" => 0x43,
            "KeyD" => 0x44,
            "KeyE" => 0x45,
            "KeyF" => 0x46,
            "KeyG" => 0x47,
            "KeyH" => 0x48,
            "KeyI" => 0x49,
            "KeyJ" => 0x4A,
            "KeyK" => 0x4B,
            "KeyL" => 0x4C,
            "KeyM" => 0x4D,
            "KeyN" => 0x4E,
            "KeyO" => 0x4F,
            "KeyP" => 0x50,
            "KeyQ" => 0x51,
            "KeyR" => 0x52,
            "KeyS" => 0x53,
            "KeyT" => 0x54,
            "KeyU" => 0x55,
            "KeyV" => 0x56,
            "KeyW" => 0x57,
            "KeyX" => 0x58,
            "KeyY" => 0x59,
            "KeyZ" => 0x5A,
            "Digit0" => 0x30,
            "Digit1" => 0x31,
            "Digit2" => 0x32,
            "Digit3" => 0x33,
            "Digit4" => 0x34,
            "Digit5" => 0x35,
            "Digit6" => 0x36,
            "Digit7" => 0x37,
            "Digit8" => 0x38,
            "Digit9" => 0x39,
            "Enter" => 0x0D,
            "Escape" => 0x1B,
            "Backspace" => 0x08,
            "Tab" => 0x09,
            "Space" => 0x20,
            "ArrowLeft" => 0x25,
            "ArrowUp" => 0x26,
            "ArrowRight" => 0x27,
            "ArrowDown" => 0x28,
            "Home" => 0x24,
            "End" => 0x23,
            "PageUp" => 0x21,
            "PageDown" => 0x22,
            "Insert" => 0x2D,
            "Delete" => 0x2E,
            "ShiftLeft" => 0xA0,
            "ShiftRight" => 0xA1,
            "ControlLeft" => 0xA2,
            "ControlRight" => 0xA3,
            "AltLeft" => 0xA4,
            "AltRight" => 0xA5,
            "F1" => 0x70,
            "F2" => 0x71,
            "F3" => 0x72,
            "F4" => 0x73,
            "F5" => 0x74,
            "F6" => 0x75,
            "F7" => 0x76,
            "F8" => 0x77,
            "F9" => 0x78,
            "F10" => 0x79,
            "F11" => 0x7A,
            "F12" => 0x7B,
            "Numpad0" => 0x60,
            "Numpad1" => 0x61,
            "Numpad2" => 0x62,
            "Numpad3" => 0x63,
            "Numpad4" => 0x64,
            "Numpad5" => 0x65,
            "Numpad6" => 0x66,
            "Numpad7" => 0x67,
            "Numpad8" => 0x68,
            "Numpad9" => 0x69,
            "NumpadMultiply" => 0x6A,
            "NumpadAdd" => 0x6B,
            "NumpadSubtract" => 0x6D,
            "NumpadDecimal" => 0x6E,
            "NumpadDivide" => 0x6F,
            "NumpadEnter" => 0x0D,
            "Semicolon" => 0xBA,
            "Equal" => 0xBB,
            "Comma" => 0xBC,
            "Minus" => 0xBD,
            "Period" => 0xBE,
            "Slash" => 0xBF,
            "Backquote" => 0xC0,
            "BracketLeft" => 0xDB,
            "Backslash" => 0xDC,
            "BracketRight" => 0xDD,
            "Quote" => 0xDE,
            _ => return None,
        };
        return Some(vk);
    }

    if let Some(rest) = v.strip_prefix("Joy") {
        let dot = rest.find('.')?;
        let port: u32 = rest[..dot].parse().ok()?;
        let control = &rest[dot + 1..];
        let base = 0x11000_u32 + port * 0x100;

        if let Some(axis_part) = control.strip_prefix("Axis") {
            let (idx_str, dir) = axis_part.split_at(axis_part.len() - 1);
            let axis_idx: u32 = idx_str.parse().ok()?;
            let di_idx: u32 = match (axis_idx, dir) {
                (0, "-") => 2,
                (0, "+") => 3,
                (1, "-") => 1,
                (1, "+") => 0,
                (2, "-") => 6,
                (2, "+") => 7,
                (3, "-") => 5,
                (3, "+") => 4,
                _ => return None,
            };
            return Some(base + di_idx);
        }

        if let Some(btn_str) = control.strip_prefix("Btn") {
            let btn_idx: u32 = btn_str.parse().ok()?;
            let di_idx: u32 = match btn_idx {
                0 => 17,
                1 => 18,
                2 => 16,
                3 => 19,
                12 => 0x0C,
                13 => 0x0D,
                14 => 0x0F,
                15 => 0x0E,
                i => 0x10 + i,
            };
            return Some(base + di_idx);
        }
    }

    None
}

fn mesen_nes_button_to_emunex_key(mesen_button: &str) -> Option<&'static str> {
    match mesen_button {
        "A" => Some("A"),
        "B" => Some("X"),
        "Select" => Some("Select"),
        "Start" => Some("Start"),
        "Up" => Some("DPad-Up"),
        "Down" => Some("DPad-Down"),
        "Left" => Some("DPad-Left"),
        "Right" => Some("DPad-Right"),
        _ => None,
    }
}

pub fn map_mesen(xml_content: &str, global_mappings: &HashMap<String, String>) -> String {
    const NES_TAGS: &[&str] = &["A", "B", "Select", "Start", "Up", "Down", "Left", "Right"];

    let mut out = String::with_capacity(xml_content.len());

    let mut in_input_device = false;
    let mut in_key_mappings = false;
    let mut device_count = 0u32;
    let mut mapping_count = 0u32;

    let bytes = xml_content.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;

    while i < len {
        if bytes[i] == b'<' {
            let mut j = i + 1;
            while j < len && bytes[j] != b'>' {
                j += 1;
            }
            let tag_slice = &xml_content[i..=j];
            let inner = xml_content[i + 1..j].trim();

            if !inner.starts_with('/') && !inner.ends_with('/') {
                if inner == "InputDevice" {
                    device_count += 1;
                    if device_count == 1 {
                        in_input_device = true;
                        mapping_count = 0;
                    }
                } else if inner == "KeyMappings" && in_input_device {
                    mapping_count += 1;
                    if mapping_count == 1 {
                        in_key_mappings = true;
                    }
                }
            }

            if inner.starts_with('/') {
                let name = inner.trim_start_matches('/');
                if name == "InputDevice" && in_input_device {
                    in_input_device = false;
                    in_key_mappings = false;
                } else if name == "KeyMappings" && in_key_mappings {
                    in_key_mappings = false;
                }
            }

            if in_key_mappings && !inner.starts_with('/') && !inner.ends_with('/') {
                if NES_TAGS.contains(&inner) {
                    let tag_name = inner;
                    out.push_str(tag_slice);
                    i = j + 1;

                    let close_pat = format!("</{}>", tag_name);
                    if let Some(close_rel) = xml_content[i..].find(&close_pat) {
                        let close_abs = i + close_rel;
                        let new_val: u32 = mesen_nes_button_to_emunex_key(tag_name)
                            .and_then(|k| global_mappings.get(k))
                            .and_then(|v| emunex_to_mesen_keycode(v))
                            .unwrap_or(0);
                        out.push_str(&new_val.to_string());
                        out.push_str(&close_pat);
                        i = close_abs + close_pat.len();
                        continue;
                    }
                    continue;
                }
            }

            out.push_str(tag_slice);
            i = j + 1;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }

    out
}
