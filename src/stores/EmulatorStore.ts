import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { ref } from "vue";
import { getGlobalStore, getSavedDomains, getDomainStore } from "../lib/store";

export type Emulator = {
    id: string;
    name: string;
    consoles: string[];
    is_default: boolean;
    binary_path: string;
    run_command: string;
    save_path?: string;
    save_extensions: string[];
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
    save_extensions: string[];
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
            const globalStore = await getGlobalStore();

            const migrated = await globalStore.get<boolean>("emulators_migrated_v2");
            if (!migrated) {
                const domains = await getSavedDomains();
                let anyMigrated = false;
                const currentGlobal = await globalStore.get<Record<string, any>>("emulators") || {};

                for (const domain of domains) {
                    const domainStore = await getDomainStore(domain);
                    const domainEms = await domainStore.get<Record<string, any>>("emulators");
                    const safeDomain = domain.replace(/[^a-z0-9]/gi, "_").toLowerCase();

                    if (domainEms && typeof domainEms === 'object') {
                        for (const [key, value] of Object.entries(domainEms)) {
                            if (Array.isArray(value)) {
                                for (const emu of value) {
                                    const newId = `server-${safeDomain}-${emu.id}`;
                                    if (!currentGlobal[newId]) {
                                        currentGlobal[newId] = { ...emu, id: newId, consoles: [(emu as any).console || key] };
                                        anyMigrated = true;
                                    } else {
                                        const c = (emu as any).console || key;
                                        if (!currentGlobal[newId].consoles.includes(c)) {
                                            currentGlobal[newId].consoles.push(c);
                                            anyMigrated = true;
                                        }
                                    }
                                }
                            } else {
                                const newId = key.startsWith("server-") ? `server-${safeDomain}-${key.slice(7)}` : key;
                                if (!currentGlobal[newId]) {
                                    currentGlobal[newId] = { ...(value as any), id: newId };
                                    anyMigrated = true;
                                }
                            }
                        }

                        await domainStore.delete("emulators");
                        await domainStore.save();
                    }
                }

                if (anyMigrated) {
                    await globalStore.set("emulators", currentGlobal);
                }

                await invoke("migrate_emulator_files");

                await globalStore.set("emulators_migrated_v2", true);
                await globalStore.save();
            }

            const ems = await globalStore.get<Record<string, any>>("emulators");
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
            e.consoles.some(c => c.toLowerCase() === normalized)
        );
    }

    async function saveEmulator(data: Emulator) {
        const store = await getGlobalStore();

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

        const store = await getGlobalStore();
        await store.set("emulators", emulators.value);
        await store.save();
    }

    async function removeEmulator(emulatorId: string) {
        loading.value = true;
        try {
            await invoke("remove_emulator", { emulatorId });
            await fetchEmulators();
        } catch (e) {
            console.error("Failed to remove emulator:", e);
        } finally {
            loading.value = false;
        }
    }

    async function fetchServerEmulators(consoleName: string): Promise<ServerEmulator[]> {
        loading.value = true;
        try {
            const results = await invoke<ServerEmulator[]>("fetch_server_emulators", { console: consoleName });
            return results;
        } catch (e) {
            return [];
        } finally {
            loading.value = false;
        }
    }

    async function downloadEmulator(consoleName: string, serverEmulatorId: string) {
        loading.value = true;
        try {
            await invoke("download_emulator", {
                console: consoleName,
                emulatorId: serverEmulatorId,
            });

            await fetchEmulators();
        } catch (e) {
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
