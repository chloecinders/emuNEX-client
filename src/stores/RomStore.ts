import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { ref } from "vue";

export interface LocalStorageEntry {
    console?: string;
    game_id: string;
    rom_size: number;
    save_size: number;
    local_version?: number;
}

export interface GameSaveFile {
    path: string;
    size: number;
    hash: string;
}

export interface SyncResult {
    conflict: boolean;
    latest_version: number | null;
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

    const getGameSaveFiles = async (gameId: string): Promise<GameSaveFile[]> => {
        try {
            return await invoke<GameSaveFile[]>("get_game_save_files", { gameId });
        } catch (e) {
            console.error("Failed to get game save files:", e);
            return [];
        }
    };

    const checkSaveStatus = async (gameId: string): Promise<SyncResult | null> => {
        try {
            return await invoke<SyncResult>("check_save_status", { gameId });
        } catch (e) {
            console.error("Failed to check save status:", e);
            return null;
        }
    };

    const openSaveFolder = async (gameId: string) => {
        try {
            await invoke("open_save_folder", { gameId });
        } catch (e) {
            console.error("Failed to open save folder:", e);
        }
    };

    return {
        installedRoms,
        loading,
        fetchInstalledRoms,
        deleteRom,
        deleteSave,
        getGameSaveFiles,
        checkSaveStatus,
        openSaveFolder,
    };
});
