import { invoke } from "@tauri-apps/api/core";
import { openUrl as tauriOpenUrl } from "@tauri-apps/plugin-opener";

export async function openExternalUrl(url: string): Promise<void> {
    try {
        await invoke("open_external_url", { url });
    } catch (e) {
        console.warn("Backend opener failed, falling back to plugin-opener:", e);
        await tauriOpenUrl(url);
    }
}
