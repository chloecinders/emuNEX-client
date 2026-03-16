import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { ref } from "vue";

export interface LocalStorageEntry {
    console?: string;
    game_id: string;
    rom_size: number;
    save_size: number;
}

export const useRomStore = defineStore("rom", () => {
    const installedRoms = ref<LocalStorageEntry[]>([]);
    const loading = ref(false);

    const fetchInstalledRoms = async () => {
        try {
            loading.value = true;
            installedRoms.value = await invoke<LocalStorageEntry[]>("get_local_storage");
        } catch (e) {
            console.error("Failed to fetch local storage entries:", e);
        } finally {
            loading.value = false;
        }
    };

    const deleteRom = async (gameId: string, consoleName: string) => {
        try {
            loading.value = true;
            await invoke("delete_installed_rom", { gameId, console: consoleName });
            await fetchInstalledRoms();
        } catch (e) {
            console.error("Failed to delete rom:", e);
            throw e;
        } finally {
            loading.value = false;
        }
    };

    const deleteSave = async (gameId: string) => {
        try {
            loading.value = true;
            await invoke("delete_local_save", { gameId });
            await fetchInstalledRoms();
        } catch (e) {
            console.error("Failed to delete save:", e);
            throw e;
        } finally {
            loading.value = false;
        }
    };

    return {
        installedRoms,
        loading,
        fetchInstalledRoms,
        deleteRom,
        deleteSave,
    };
});
