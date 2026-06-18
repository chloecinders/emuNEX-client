use std::collections::HashMap;

fn emunex_to_sdl_button(emunex_mapping: &str) -> Option<u32> {
    if let Some(rest) = emunex_mapping.strip_prefix("Joy") {
        let dot = rest.find('.')?;
        let control = &rest[dot + 1..];

        if let Some(btn_str) = control.strip_prefix("Btn") {
            let btn_idx: u32 = btn_str.parse().ok()?;
            return match btn_idx {
                0 => Some(0),
                1 => Some(1),
                2 => Some(2),
                3 => Some(3),
                4 => Some(9),
                5 => Some(10),
                8 => Some(4),
                9 => Some(6),
                10 => Some(7),
                11 => Some(8),
                12 => Some(11),
                13 => Some(12),
                14 => Some(13),
                15 => Some(14),
                _ => None,
            };
        }
    }
    None
}

fn emunex_to_mgba_axis(emunex_mapping: &str) -> Option<(i32, i32)> {
    if let Some(rest) = emunex_mapping.strip_prefix("Joy") {
        let dot = rest.find('.')?;
        let control = &rest[dot + 1..];

        if let Some(axis_str) = control.strip_prefix("Axis") {
            let (idx_str, dir) = axis_str.split_at(axis_str.len() - 1);
            let axis_idx: i32 = idx_str.parse().ok()?;
            let dir_sign = if dir == "+" { 1 } else { -1 };
            return Some((axis_idx, dir_sign));
        }
    }
    None
}

fn emunex_keyboard_to_qt_key(emunex_mapping: &str) -> Option<u32> {
    if let Some(code) = emunex_mapping.strip_prefix("Kbd.") {
        if code.starts_with("Key") {
            let ch = code.chars().nth(3)?;
            return Some(ch.to_ascii_uppercase() as u32);
        }
        if code.starts_with("Digit") {
            let ch = code.chars().nth(5)?;
            return Some(ch as u32);
        }
        return match code {
            "Enter" => Some(16777220),
            "Escape" => Some(16777216),
            "Backspace" => Some(16777219),
            "Tab" => Some(16777217),
            "Space" => Some(0x20),
            "ArrowLeft" => Some(16777234),
            "ArrowUp" => Some(16777235),
            "ArrowRight" => Some(16777236),
            "ArrowDown" => Some(16777237),
            "ShiftLeft" | "ShiftRight" => Some(16777248),
            "ControlLeft" | "ControlRight" => Some(16777249),
            "AltLeft" | "AltRight" => Some(16777251),
            _ => None,
        };
    }
    None
}

macro_rules! flush_missing {
    ($out:expr, $sec_type:expr, $handled:expr, $global_mappings:expr, $gba_keys:expr) => {
        if $sec_type != 0 {
            for &(mgba_k, emunex_k, mgba_axis) in $gba_keys.iter() {
                if !$handled.contains(mgba_k) {
                    if let Some(emu_val) = $global_mappings.get(emunex_k) {
                        if $sec_type == 1 {
                            if let Some(sdl_btn) = emunex_to_sdl_button(emu_val) {
                                $out.push_str(&format!("{}={}\n", mgba_k, sdl_btn));
                            } else if let Some((axis_idx, dir_sign)) = emunex_to_mgba_axis(emu_val)
                            {
                                let axis_key = format!("{}Axis", mgba_axis);
                                let val_key = format!("{}Value", mgba_axis);
                                let dir_char = if dir_sign > 0 { '+' } else { '-' };
                                $out.push_str(&format!("{}={}{}\n", axis_key, dir_char, axis_idx));
                                let val = if dir_sign > 0 { 16384 } else { -16384 };
                                $out.push_str(&format!("{}={}\n", val_key, val));
                            }
                        } else if $sec_type == 2 {
                            if let Some(qt_key) = emunex_keyboard_to_qt_key(emu_val) {
                                $out.push_str(&format!("{}={}\n", mgba_k, qt_key));
                            }
                        }
                    }
                }
            }
            $handled.clear();
        }
    };
}

pub fn map_mgba(ini_content: &str, global_mappings: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(ini_content.len());
    let mut current_section_type = 0;
    let mut current_section_handled_keys = std::collections::HashSet::new();

    let gba_keys = [
        ("keyA", "A", "axisA"),
        ("keyB", "X", "axisB"),
        ("keyL", "L", "axisL"),
        ("keyR", "R", "axisR"),
        ("keyStart", "Start", "axisStart"),
        ("keySelect", "Select", "axisSelect"),
        ("keyUp", "DPad-Up", "axisUp"),
        ("keyDown", "DPad-Down", "axisDown"),
        ("keyLeft", "DPad-Left", "axisLeft"),
        ("keyRight", "DPad-Right", "axisRight"),
    ];

    for line in ini_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            flush_missing!(
                &mut out,
                current_section_type,
                current_section_handled_keys,
                global_mappings,
                gba_keys
            );

            if trimmed.starts_with("[gba.input.SDL")
                || trimmed.starts_with("[gba.input-profile.")
                || trimmed.starts_with("[gba.input.XInput")
            {
                current_section_type = 1;
            } else if trimmed.starts_with("[gba.input.QT_K]") {
                current_section_type = 2;
            } else {
                current_section_type = 0;
            }
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if current_section_type > 0 {
            if let Some((k, _)) = line.split_once('=') {
                let key_name = k.trim();
                let mut handled_line = false;

                if current_section_type == 1 && key_name.starts_with("hat") {
                    continue;
                }

                for &(mgba_k, emunex_k, mgba_axis) in gba_keys.iter() {
                    let axis_key = format!("{}Axis", mgba_axis);
                    let val_key = format!("{}Value", mgba_axis);

                    if key_name == mgba_k {
                        if current_section_type == 1 {
                            if let Some(emu_val) = global_mappings.get(emunex_k) {
                                if let Some(sdl_btn) = emunex_to_sdl_button(emu_val) {
                                    out.push_str(&format!("{}={}\n", key_name, sdl_btn));
                                    current_section_handled_keys.insert(mgba_k);
                                    handled_line = true;
                                    break;
                                } else if emunex_to_mgba_axis(emu_val).is_some() {
                                    current_section_handled_keys.insert(mgba_k);
                                    handled_line = true;
                                    break;
                                }
                            }
                        } else if current_section_type == 2 {
                            if let Some(emu_val) = global_mappings.get(emunex_k) {
                                if let Some(qt_key) = emunex_keyboard_to_qt_key(emu_val) {
                                    out.push_str(&format!("{}={}\n", key_name, qt_key));
                                    current_section_handled_keys.insert(mgba_k);
                                    handled_line = true;
                                    break;
                                }
                            }
                        }
                    }

                    if current_section_type == 1 && (key_name == &axis_key || key_name == &val_key)
                    {
                        if let Some(emu_val) = global_mappings.get(emunex_k) {
                            if let Some((axis_idx, dir_sign)) = emunex_to_mgba_axis(emu_val) {
                                if key_name == &axis_key {
                                    let dir_char = if dir_sign > 0 { '+' } else { '-' };
                                    out.push_str(&format!(
                                        "{}={}{}\n",
                                        axis_key, dir_char, axis_idx
                                    ));
                                } else if key_name == &val_key {
                                    let val = if dir_sign > 0 { 16384 } else { -16384 };
                                    out.push_str(&format!("{}={}\n", val_key, val));
                                }
                                current_section_handled_keys.insert(mgba_k);
                                handled_line = true;
                                break;
                            } else {
                                current_section_handled_keys.insert(mgba_k);
                                handled_line = true;
                                break;
                            }
                        }
                    }
                }
                if handled_line {
                    continue;
                }
            }
        }

        out.push_str(line);
        out.push('\n');
    }

    flush_missing!(
        &mut out,
        current_section_type,
        current_section_handled_keys,
        global_mappings,
        gba_keys
    );

    out
}
