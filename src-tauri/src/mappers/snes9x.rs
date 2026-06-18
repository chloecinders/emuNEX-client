use std::collections::HashMap;

fn emunex_keyboard_to_snes9x(v: &str) -> Option<String> {
    if let Some(code) = v.strip_prefix("Kbd.") {
        let mapped = match code {
            "Space" => "Space",
            "Enter" => "Enter",
            "Escape" => "Esc",
            "Backspace" => "Backspace",
            "Tab" => "Tab",
            "ShiftLeft" | "ShiftRight" => "Shift",
            "ControlLeft" | "ControlRight" => "Ctrl",
            "AltLeft" | "AltRight" => "Alt",
            "ArrowLeft" => "Left",
            "ArrowRight" => "Right",
            "ArrowUp" => "Up",
            "ArrowDown" => "Down",
            "PageUp" => "PgUp",
            "PageDown" => "PgDn",
            "End" => "End",
            "Home" => "Home",
            "Insert" => "Insert",
            "Delete" => "Delete",
            "NumpadAdd" => "+",
            "NumpadSubtract" => "-",
            c if c.starts_with("Key") => &c[3..],
            c if c.starts_with("Digit") => &c[5..],
            c if c.starts_with("F") => c,
            _ => "Unassigned",
        };
        return Some(mapped.to_string());
    }
    None
}

fn emunex_joypad_to_snes9x(v: &str, prefix: &str) -> Option<String> {
    if let Some(rest) = v.strip_prefix("Joy") {
        let dot = rest.find('.')?;
        let control = &rest[dot + 1..];

        if let Some(btn_str) = control.strip_prefix("Btn") {
            let btn_idx: u32 = btn_str.parse().ok()?;
            return match btn_idx {
                0 => Some(format!("{}Button 1", prefix)),
                1 => Some(format!("{}Button 2", prefix)),
                2 => Some(format!("{}Button 0", prefix)),
                3 => Some(format!("{}Button 3", prefix)),

                12 => Some(format!("{}POV Up", prefix)),
                13 => Some(format!("{}POV Down", prefix)),
                14 => Some(format!("{}POV Left", prefix)),
                15 => Some(format!("{}POV Right", prefix)),
                _ => Some(format!("{}Button {}", prefix, btn_idx)),
            };
        } else if let Some(axis_str) = control.strip_prefix("Axis") {
            let (idx_str, dir) = axis_str.split_at(axis_str.len() - 1);
            let axis_idx: u32 = idx_str.parse().ok()?;
            let axis_name = match axis_idx {
                0 => "X",
                1 => "Y",
                2 => "Z",
                3 => "Rz",
                4 => "Slider",
                _ => return Some(format!("{}Axis {}{}", prefix, axis_idx, dir)),
            };
            return Some(format!("{}Axis {}{}", prefix, axis_name, dir));
        }
    }
    None
}

fn emunex_to_snes9x(v: &str, prefix: &str) -> String {
    if v == "Unmapped" || v == "..." || v.is_empty() {
        return "Unassigned".to_string();
    }
    if let Some(res) = emunex_keyboard_to_snes9x(v) {
        return res;
    }
    if let Some(res) = emunex_joypad_to_snes9x(v, prefix) {
        return res;
    }
    "Unassigned".to_string()
}

fn get_joypad_prefix(config_content: &str) -> String {
    for line in config_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Joypad1:") {
            if let Some(eq) = trimmed.find('=') {
                let val = trimmed[eq + 1..].trim();
                if val.starts_with("(J") {
                    if let Some(end) = val.find(')') {
                        return val[..=end].to_string();
                    }
                }
            }
        }
    }
    "(J1)".to_string()
}

pub fn map_snes9x(config_content: &str, global_mappings: &HashMap<String, String>) -> String {
    let mut out_lines = Vec::new();

    let jd_prefix = get_joypad_prefix(config_content);

    let target_keys = std::collections::HashMap::from([
        ("Joypad1:A", "B"),
        ("Joypad1:B", "A"),
        ("Joypad1:X", "Y"),
        ("Joypad1:Y", "X"),
        ("Joypad1:L", "L"),
        ("Joypad1:R", "R"),
        ("Joypad1:Select", "Select"),
        ("Joypad1:Start", "Start"),
        ("Joypad1:Up", "DPad-Up"),
        ("Joypad1:Down", "DPad-Down"),
        ("Joypad1:Left", "DPad-Left"),
        ("Joypad1:Right", "DPad-Right"),
    ]);

    for line in config_content.lines() {
        if let Some(eq_idx) = line.find('=') {
            let left_part = &line[..eq_idx];
            let key = left_part.trim();
            if let Some(global_key) = target_keys.get(key) {
                if let Some(mapping) = global_mappings.get(*global_key) {
                    let new_val = emunex_to_snes9x(mapping, &jd_prefix);

                    let before_eq = line[..eq_idx].to_string();
                    let right_part = &line[eq_idx + 1..];
                    let comment = right_part.find('#').map(|i| &right_part[i..]).unwrap_or("");

                    let mut final_right = new_val;
                    if !comment.is_empty() {
                        let pad_len = 28_usize.saturating_sub(final_right.len());
                        let padding = " ".repeat(std::cmp::max(1, pad_len));
                        final_right.push_str(&padding);
                        final_right.push_str(comment);
                    }

                    out_lines.push(format!("{}={}", before_eq, final_right));
                    continue;
                }
            }
        }
        out_lines.push(line.to_string());
    }

    out_lines.join("\n")
}
