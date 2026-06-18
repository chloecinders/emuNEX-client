use std::collections::{HashMap, HashSet};

fn get_p64_device_prefix_from_config(config_content: &str) -> Option<String> {
    for line in config_content.lines() {
        let trimmed = line.trim();
        let Some(eq) = trimmed.find('=') else {
            continue;
        };
        let key = trimmed[..eq].trim();
        let is_input_key = key.starts_with("Button")
            || key.starts_with("Analog")
            || key.starts_with("DPad")
            || key.starts_with("CButton");
        if !is_input_key {
            continue;
        }
        let val = trimmed[eq + 1..].trim();
        if let Some(prefix) = val.split_whitespace().next() {
            if prefix.starts_with('{') && prefix.ends_with('}') {
                return Some(prefix.to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn get_first_joystick_guid_from_system() -> Option<String> {
    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Copy, Clone)]
    struct GUID {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    #[repr(C)]
    struct DIDEVICEINSTANCEW {
        dw_size: u32,
        guid_instance: GUID,
    }

    const IID_IDIRECTINPUT8W: GUID = GUID {
        data1: 0xBF798031,
        data2: 0x483A,
        data3: 0x4DA2,
        data4: [0xAA, 0x99, 0x5D, 0x64, 0xED, 0x36, 0x97, 0x00],
    };
    const DIRECTINPUT_VERSION: u32 = 0x0800;
    const DI8DEVCLASS_GAMECTRL: u32 = 4;
    const DIEDFL_ATTACHEDONLY: u32 = 0x00000001;

    #[link(name = "dinput8")]
    extern "system" {
        fn DirectInput8Create(
            hinst: *mut c_void,
            dw_version: u32,
            riid: *const GUID,
            ppv_out: *mut *mut c_void,
            punk_outer: *mut c_void,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(lp_module_name: *const u16) -> *mut c_void;
    }

    unsafe extern "system" fn enum_callback(
        lpddi: *const DIDEVICEINSTANCEW,
        pv_ref: *mut c_void,
    ) -> i32 {
        if !lpddi.is_null() {
            *(pv_ref as *mut Option<GUID>) = Some((*lpddi).guid_instance);
        }
        0
    }

    unsafe {
        let hinst = GetModuleHandleW(std::ptr::null());
        let mut di_raw: *mut c_void = std::ptr::null_mut();

        let hr = DirectInput8Create(
            hinst,
            DIRECTINPUT_VERSION,
            &IID_IDIRECTINPUT8W,
            &mut di_raw,
            std::ptr::null_mut(),
        );

        if hr != 0 || di_raw.is_null() {
            return None;
        }

        let vtbl = *(di_raw as *mut *mut *const c_void);

        type EnumFn = unsafe extern "system" fn(
            *mut c_void,
            u32,
            unsafe extern "system" fn(*const DIDEVICEINSTANCEW, *mut c_void) -> i32,
            *mut c_void,
            u32,
        ) -> i32;
        type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;

        let enum_devices: EnumFn = std::mem::transmute(*vtbl.add(4));
        let release: ReleaseFn = std::mem::transmute(*vtbl.add(2));

        let mut found: Option<GUID> = None;
        enum_devices(
            di_raw,
            DI8DEVCLASS_GAMECTRL,
            enum_callback,
            &mut found as *mut Option<GUID> as *mut c_void,
            DIEDFL_ATTACHEDONLY,
        );

        release(di_raw);

        found.map(|g| {
            format!(
                "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
                g.data1,
                g.data2,
                g.data3,
                g.data4[0],
                g.data4[1],
                g.data4[2],
                g.data4[3],
                g.data4[4],
                g.data4[5],
                g.data4[6],
                g.data4[7],
            )
        })
    }
}

#[cfg(not(target_os = "windows"))]
fn get_first_joystick_guid_from_system() -> Option<String> {
    None
}

fn get_p64_device_prefix(config_content: &str) -> Option<String> {
    get_p64_device_prefix_from_config(config_content).or_else(get_first_joystick_guid_from_system)
}

fn emunex_to_p64(v: &str, device_prefix: &str) -> Option<String> {
    let rest = v.strip_prefix("Joy")?;
    let dot = rest.find('.')?;
    let control = &rest[dot + 1..];

    if let Some(btn_str) = control.strip_prefix("Btn") {
        let btn_idx: u32 = btn_str.parse().ok()?;
        return Some(match btn_idx {
            12 => format!("{} 08 3 3", device_prefix),
            13 => format!("{} 08 1 3", device_prefix),
            14 => format!("{} 08 0 3", device_prefix),
            15 => format!("{} 08 2 3", device_prefix),
            n => format!("{} {:02} 0 1", device_prefix, n),
        });
    }

    if let Some(axis_str) = control.strip_prefix("Axis") {
        if axis_str.is_empty() {
            return None;
        }
        let (idx_str, dir_char) = axis_str.split_at(axis_str.len() - 1);
        let axis_idx: u32 = idx_str.parse().ok()?;
        let dir = if dir_char == "+" { "1" } else { "0" };
        return Some(format!("{} {:02} {} 2", device_prefix, axis_idx, dir));
    }

    None
}

fn flush_missing(
    target_keys: &HashMap<&str, &str>,
    handled: &HashSet<String>,
    device_prefix: &str,
    global_mappings: &HashMap<String, String>,
    out_lines: &mut Vec<String>,
) {
    let mut missing: Vec<(&str, &str)> = target_keys
        .iter()
        .filter(|(k, _)| !handled.contains(**k))
        .map(|(&k, &v)| (k, v))
        .collect();
    missing.sort_by_key(|(k, _)| *k);

    for (p64_key, global_key) in missing {
        if let Some(mapping) = global_mappings.get(global_key) {
            if let Some(val) = emunex_to_p64(mapping, device_prefix) {
                out_lines.push(format!("{}={}", p64_key, val));
            }
        }
    }
}

pub fn map_project64(config_content: &str, global_mappings: &HashMap<String, String>) -> String {
    let Some(device_prefix) = get_p64_device_prefix(config_content) else {
        return config_content.to_string();
    };

    let target_keys: HashMap<&str, &str> = HashMap::from([
        ("ButtonA", "A"),
        ("ButtonB", "X"),
        ("ButtonZ", "ZL"),
        ("ButtonL", "L"),
        ("ButtonR", "R"),
        ("ButtonStart", "Start"),
        ("AnalogUp", "LS Up"),
        ("AnalogDown", "LS Down"),
        ("AnalogLeft", "LS Left"),
        ("AnalogRight", "LS Right"),
        ("CButtonUp", "RS Up"),
        ("CButtonDown", "RS Down"),
        ("CButtonLeft", "RS Left"),
        ("CButtonRight", "RS Right"),
        ("DPadUp", "DPad-Up"),
        ("DPadDown", "DPad-Down"),
        ("DPadLeft", "DPad-Left"),
        ("DPadRight", "DPad-Right"),
    ]);

    let mut out_lines: Vec<String> = Vec::new();
    let mut in_section = false;
    let mut section_found = false;
    let mut handled: HashSet<String> = HashSet::new();

    for line in config_content.lines() {
        let trimmed = line.trim();

        if trimmed == "[Input-Controller 1]" {
            in_section = true;
            section_found = true;
            out_lines.push(line.to_string());
            continue;
        }

        if trimmed.starts_with('[') && in_section {
            flush_missing(
                &target_keys,
                &handled,
                &device_prefix,
                global_mappings,
                &mut out_lines,
            );
            in_section = false;
            out_lines.push(line.to_string());
            continue;
        }

        if in_section {
            if let Some(eq) = line.find('=') {
                let key = line[..eq].trim();
                if let Some(&global_key) = target_keys.get(key) {
                    handled.insert(key.to_string());
                    if let Some(mapping) = global_mappings.get(global_key) {
                        if let Some(val) = emunex_to_p64(mapping, &device_prefix) {
                            out_lines.push(format!("{}={}", key, val));
                            continue;
                        }
                    }
                }
            }
        }

        out_lines.push(line.to_string());
    }

    if in_section {
        flush_missing(
            &target_keys,
            &handled,
            &device_prefix,
            global_mappings,
            &mut out_lines,
        );
    }

    if !section_found {
        if out_lines.last().map_or(true, |l| !l.is_empty()) {
            out_lines.push(String::new());
        }
        out_lines.push("[Input-Controller 1]".to_string());
        flush_missing(
            &target_keys,
            &HashSet::new(),
            &device_prefix,
            global_mappings,
            &mut out_lines,
        );
    }

    out_lines.join("\n")
}
