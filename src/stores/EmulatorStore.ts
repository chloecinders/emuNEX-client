import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { ref } from "vue";
import { getStore } from "../lib/store";

export type Emulator = {
    id: string;
    name: string;
    is_default: boolean;
    is_installed: boolean;
    binary_path: string;
    run_command: string;
    save_path?: string;
    config_files: string[];
    zipped: boolean;
};

export type ServerEmulator = {
    id: string;
    name: string;
    console: string;
    platform: string;
    run_command: string;
    binary_path: string;
    binary_name: string | null;
    save_path: string | null;
    config_files: string[];
    zipped: boolean;
    file_size: number;
    md5_hash: string | null;
};

export const useEmulatorStore = defineStore("emulatorStore", () => {
    const loading = ref(false);
    const emulators = ref<Record<string, Emulator[]>>({});

    async function fetchEmulators() {
        loading.value = true;
        try {
            const store = await getStore();
            const ems = await store.get<Record<string, any>>("emulators");

            const validEms: Record<string, Emulator[]> = {};
            if (ems && typeof ems === 'object') {
                for (const [key, value] of Object.entries(ems)) {
                    if (Array.isArray(value)) {
                        validEms[key.toLowerCase()] = value as Emulator[];
                    }
                }
            }

            emulators.value = validEms;
        } catch (err) {
            console.error("Failed to fetch emulators:", err);
        } finally {
            loading.value = false;
        }
    }

    async function isEmulatorInstalled(cs: string): Promise<boolean> {
        const normalized = cs.toLowerCase();
        if (Object.keys(emulators.value).length === 0) await fetchEmulators();
        const list = emulators.value[normalized];
        if (!Array.isArray(list)) return false;
        return list.some(e => e.is_installed);
    }

    async function saveEmulator(consoleName: string, data: Emulator) {
        const normalized = consoleName.toLowerCase();
        const store = await getStore();
        if (!emulators.value[normalized]) {
            emulators.value[normalized] = [];
        }

        const list = emulators.value[normalized];
        const existingIndex = list.findIndex((e) => e.id === data.id);

        if (existingIndex > -1) {
            list[existingIndex] = { ...data };
        } else {
            if (list.length === 0) data.is_default = true;
            list.push(data);
        }

        await store.set("emulators", emulators.value);
        await store.save();
    }

    async function setDefaultEmulator(consoleName: string, emulatorId: string) {
        const normalized = consoleName.toLowerCase();
        const list = emulators.value[normalized];
        if (!list) return;

        list.forEach(e => {
            e.is_default = e.id === emulatorId;
        });

        const store = await getStore();
        await store.set("emulators", emulators.value);
        await store.save();
    }

    async function removeEmulator(consoleName: string, emulatorId: string) {
        const normalized = consoleName.toLowerCase();
        const list = emulators.value[normalized];
        if (!list) return;

        const filtered = list.filter((e) => e.id !== emulatorId);

        if (filtered.length > 0 && !filtered.some(e => e.is_default)) {
            filtered[0].is_default = true;
        }

        emulators.value[normalized] = filtered;

        const store = await getStore();
        await store.set("emulators", emulators.value);
        await store.save();
    }

    async function fetchServerEmulators(consoleName: string): Promise<ServerEmulator[]> {
        loading.value = true;
        try {
            return await invoke<ServerEmulator[]>("fetch_server_emulators", { console: consoleName });
        } catch (e) {
            console.error("Failed to fetch server emulators:", e);
            return [];
        } finally {
            loading.value = false;
        }
    }

    async function downloadEmulator(consoleName: string, serverEmulatorId: string) {
        const normalized = consoleName.toLowerCase();
        loading.value = true;
        try {
            await invoke("download_emulator", {
                console: normalized,
                emulatorId: serverEmulatorId,
            });
            await fetchEmulators();
        } catch (e) {
            console.error("Failed to download emulator:", e);
            throw e;
        } finally {
            loading.value = false;
        }
    }

    async function fetchAllServerEmulators(): Promise<ServerEmulator[]> {
        loading.value = true;
        try {
            return await invoke<ServerEmulator[]>("fetch_all_server_emulators");
        } catch (e) {
            console.error("Failed to fetch all server emulators:", e);
            return [];
        } finally {
            loading.value = false;
        }
    }

    return {
        emulators,
        loading,
        fetchEmulators,
        isEmulatorInstalled,
        saveEmulator,
        setDefaultEmulator,
        removeEmulator,
        fetchServerEmulators,
        downloadEmulator,
        fetchAllServerEmulators
    };
});
