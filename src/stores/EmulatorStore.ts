import { load } from "@tauri-apps/plugin-store";
import { defineStore } from "pinia";
import { Ref, ref } from "vue";

export type Emulator = {
    is_installed: boolean,
    binary_path: string,
    run_command: string
};

export const useEmulatorStore = defineStore("emulatorStore", () => {
    const loading = ref(true);
    const emulators: Ref<Record<string, Emulator>> = ref({});

    async function getEmulators(): Promise<Record<string, Emulator>> {
        const store = await load("store.json");
        const emulators = await store.get<Record<string, Emulator>>("emulators");
        loading.value = false;
        return emulators || {};
    }

    async function isEmulatorInstalled(cs: string): Promise<boolean> {
        const emulators = await getEmulators();
        return emulators[cs]?.is_installed;
    }

    getEmulators();

    return { emulators, loading, getEmulators, isEmulatorInstalled };
});
