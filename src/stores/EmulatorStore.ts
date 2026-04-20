import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { ref } from "vue";
import { getDomainStore, getGlobalStore, getSavedDomains } from "../lib/store";

export type Emulator = {
    id: string;
    name: string;
    consoles: string[];
    is_default: boolean;
    binary_path: string;
    run_command: string;
    save_path?: string;
    save_extensions: string[];
    input_config_file: string;
    input_mapper: string;
    zipped: boolean;
    file_size?: number;
    version?: string;
    source_server?: string;
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
    input_config_file: string;
    input_mapper: string;
    zipped: boolean;
    file_size: number;
    md5_hash: string | null;
    version: string | null;
    source_server?: string;
};

export const useEmulatorStore = defineStore("emulatorStore", () => {
    const loading = ref(false);
    const checkingForUpdates = ref(false);
    const emulators = ref<Record<string, Emulator>>({});
    const updatesAvailable = ref<Record<string, ServerEmulator>>({});

    async function runMigrations() {
        const globalStore = await getGlobalStore();
        if (await globalStore.get<boolean>("emulators_migrated_v2")) return;

        const domains = await getSavedDomains();
        const currentGlobal = (await globalStore.get<Record<string, any>>("emulators")) || {};
        let anyMigrated = false;

        for (const domain of domains) {
            const domainStore = await getDomainStore(domain);
            const domainEms = await domainStore.get<Record<string, any>>("emulators");

            if (!domainEms || typeof domainEms !== "object") continue;

            const safeDomain = domain.replace(/[^a-z0-9]/gi, "_").toLowerCase();

            for (const [key, value] of Object.entries(domainEms)) {
                if (!Array.isArray(value)) {
                    const newId = key.startsWith("server-") ? `server-${safeDomain}-${key.slice(7)}` : key;

                    if (!currentGlobal[newId]) {
                        currentGlobal[newId] = { ...(value as any), id: newId };
                        anyMigrated = true;
                    }

                    continue;
                }

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
            }

            await domainStore.delete("emulators");
            await domainStore.save();
        }

        if (anyMigrated) await globalStore.set("emulators", currentGlobal);

        await invoke("migrate_emulator_files");
        await globalStore.set("emulators_migrated_v2", true);
        await globalStore.save();
    }

    function parseStoredEmulators(ems: Record<string, any>): Record<string, Emulator> {
        const validEms: Record<string, Emulator> = {};

        for (const [key, value] of Object.entries(ems)) {
            if (!Array.isArray(value)) {
                validEms[key] = value as Emulator;
                continue;
            }

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
        }
        return validEms;
    }

    async function backfillMetadata(validEms: Record<string, Emulator>) {
        const domains = await getSavedDomains();
        const encodedDomains = domains.map((d) => ({
            original: d,
            encoded: d.replace(/[^a-z0-9]/gi, "_").toLowerCase(),
        }));

        let changed = false;

        for (const emu of Object.values(validEms)) {
            let emuChanged = false;

            if (emu.id.startsWith("server-") && !emu.source_server) {
                const match = encodedDomains.find((ed) => emu.id.startsWith(`server-${ed.encoded}-`));

                if (match) {
                    emu.source_server = match.original;
                    emuChanged = true;
                }
            }

            if (!emu.version) {
                emu.version = "0";
                emuChanged = true;
            }

            if (emuChanged) changed = true;
        }

        if (changed) {
            const globalStore = await getGlobalStore();
            await globalStore.set("emulators", validEms);
            await globalStore.save();
        }
    }

    async function fetchEmulators() {
        try {
            await runMigrations();
            const globalStore = await getGlobalStore();
            const ems = (await globalStore.get<Record<string, any>>("emulators")) || {};
            const validEms = parseStoredEmulators(ems);
            await backfillMetadata(validEms);
            emulators.value = validEms;
        } catch (err) {
            console.error("Failed to fetch emulators:", err);
        }
    }

    async function isEmulatorInstalled(cs: string): Promise<boolean> {
        if (Object.keys(emulators.value).length === 0) await fetchEmulators();
        return Object.values(emulators.value).some(e =>
            e.consoles.some(c => c === cs)
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

    async function downloadEmulator(consoleName: string, serverEmulatorId: string, keepConfig: boolean = false, sourceServer?: string) {
        loading.value = true;
        try {
            await invoke("download_emulator", {
                console: consoleName,
                emulator_id: serverEmulatorId,
                keep_config: keepConfig,
                source_server: sourceServer || null
            });

            await fetchEmulators();
        } catch (e) {
            throw e;
        } finally {
            loading.value = false;
        }
    }

    async function fetchAllServerEmulators(): Promise<ServerEmulator[]> {
        checkingForUpdates.value = true;
        try {
            return await invoke<ServerEmulator[]>("fetch_all_server_emulators");
        } catch (e) {
            console.error("Failed to fetch all server emulators:", e);
            return [];
        } finally {
            checkingForUpdates.value = false;
        }
    }

    async function refreshEmulatorConfig(emulatorId: string) {
        try {
            await invoke("refresh_emulator_config", { emulatorId });
            await fetchEmulators();
        } catch (e) {
            throw e;
        } finally {
            loading.value = false;
        }
    }

    async function checkForUpdates() {
        try {
            const serverList = await fetchAllServerEmulators();
            const updates: Record<string, ServerEmulator> = {};
            for (const serverEmu of serverList) {
                if (!serverEmu.source_server) continue;

                const safeDomain = serverEmu.source_server.replace(/[^a-z0-9]/gi, "_").toLowerCase();
                const localId = `server-${safeDomain}-${serverEmu.id}`;

                const local = emulators.value[localId];
                if (local) {
                    if (serverEmu.version !== null && serverEmu.version !== local.version) {
                        updates[localId] = serverEmu;
                    }
                }
            }
            updatesAvailable.value = updates;
        } catch (e) {
            console.error("Failed to check for emulator updates:", e);
        }
    }

    return {
        emulators,
        loading,
        checkingForUpdates,
        updatesAvailable,
        fetchEmulators,
        isEmulatorInstalled,
        saveEmulator,
        setDefaultEmulator,
        removeEmulator,
        fetchServerEmulators,
        downloadEmulator,
        fetchAllServerEmulators,
        refreshEmulatorConfig,
        checkForUpdates
    };
});
