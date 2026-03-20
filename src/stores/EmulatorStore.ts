import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { ref } from "vue";
import { getStore } from "../lib/store";

export type Emulator = {
    id: string;
    name: string;
    consoles: string[];
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
    consoles: string[];
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
    const emulators = ref<Record<string, Emulator>>({});

    async function fetchEmulators() {
        loading.value = true;
        try {
            const store = await getStore();
            const ems = await store.get<Record<string, any>>("emulators");

            const validEms: Record<string, Emulator> = {};
            if (ems && typeof ems === 'object') {
                for (const [key, value] of Object.entries(ems)) {
                    if (Array.isArray(value)) {
                        for (const emu of value) {
                            if (!validEms[emu.id]) {
                                validEms[emu.id] = { ...emu, consoles: [(emu as any).console || key] };
                            } else {
                                const c = (emu as any).console || key;
                                if (!validEms[emu.id].consoles.includes(c)) {
                                    validEms[emu.id].consoles.push(c);
                                }
                            }
                        }
                    } else {
                        validEms[key] = value as Emulator;
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
        return Object.values(emulators.value).some(e => 
            e.is_installed && e.consoles.some(c => c.toLowerCase() === normalized)
        );
    }

    async function saveEmulator(data: Emulator) {
        const store = await getStore();
        
        const isFirst = Object.keys(emulators.value).length === 0;
        if (isFirst && !data.id.startsWith("custom-")) data.is_default = true;

        emulators.value[data.id] = data;

        await store.set("emulators", emulators.value);
        await store.save();
    }

    async function setDefaultEmulator(emulatorId: string) {
        Object.values(emulators.value).forEach(e => {
            e.is_default = e.id === emulatorId;
        });

        const store = await getStore();
        await store.set("emulators", emulators.value);
        await store.save();
    }

    async function removeEmulator(emulatorId: string) {
        delete emulators.value[emulatorId];

        const remaining = Object.values(emulators.value);
        if (remaining.length > 0 && !remaining.some(e => e.is_default)) {
            remaining[0].is_default = true;
        }

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
