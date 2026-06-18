use std::collections::HashMap;

pub fn map_vbam(config_content: &str, global_mappings: &HashMap<String, String>) -> String {
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
