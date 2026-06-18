use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::ffi::c_void;

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
extern "system" {
    fn SHGetKnownFolderPath(
        rfid: *const [u8; 16],
        flags: u32,
        token: *mut c_void,
        path: *mut *mut u16,
    ) -> i32;
}

#[cfg(target_os = "windows")]
#[link(name = "ole32")]
extern "system" {
    fn CoTaskMemFree(pv: *mut c_void);
}

#[derive(Default, Clone, Debug)]
pub struct TemplateContext {
    pub emu_dir: String,
    pub data_dir: String,
    pub save_dir: String,
    pub temp_dir: String,
    pub rom_name: String,
    pub game_id: String,
    pub console: String,
    pub rom_path: String,
    pub bin_path: String,
    pub documents_dir: String,
}

impl TemplateContext {
    pub fn resolve(&self, template: &str) -> String {
        template
            .replace("$rom_name", &self.rom_name)
            .replace("$game_id", &self.game_id)
            .replace("$console", &self.console)
            .replace("$data_dir", &self.data_dir)
            .replace("$save_dir", &self.save_dir)
            .replace("$temp_dir", &self.temp_dir)
            .replace("$emu_dir", &self.emu_dir)
            .replace("$documents", &self.documents_dir)
            .replace("$rom", &self.rom_path)
            .replace("$bin", &self.bin_path)
            .replace("$exe", &self.bin_path)
    }

    pub fn resolve_path(&self, template: &str) -> PathBuf {
        let resolved = self.resolve(template);
        expand_env_vars(&resolved)
    }
}

pub fn get_documents_dir() -> String {
    #[cfg(target_os = "windows")]
    {
        const FOLDERID_DOCUMENTS: [u8; 16] = [
            0xD0, 0x9A, 0xD3, 0xFD, 0x8F, 0x23, 0xAF, 0x46, 0xAD, 0xB4, 0x6C, 0x85, 0x48, 0x03,
            0x69, 0xC7,
        ];

        unsafe {
            let mut path_ptr: *mut u16 = std::ptr::null_mut();
            let hr =
                SHGetKnownFolderPath(&FOLDERID_DOCUMENTS, 0, std::ptr::null_mut(), &mut path_ptr);
            if hr == 0 && !path_ptr.is_null() {
                let mut len = 0usize;
                while *path_ptr.add(len) != 0 {
                    len += 1;
                }
                let slice = std::slice::from_raw_parts(path_ptr, len);
                let result = String::from_utf16_lossy(slice);
                CoTaskMemFree(path_ptr as *mut c_void);
                return result;
            }
        }
        String::new()
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Documents")
                .to_string_lossy()
                .to_string();
        }
        String::new()
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DOCUMENTS_DIR") {
            if !xdg.is_empty() {
                return xdg;
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Documents")
                .to_string_lossy()
                .to_string();
        }
        String::new()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        String::new()
    }
}

pub fn expand_env_vars(path: &str) -> PathBuf {
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

    PathBuf::from(result)
}
