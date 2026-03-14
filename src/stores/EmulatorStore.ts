import { defineStore } from "pinia";
import { ref } from "vue";
import { getStore } from "../lib/store";

export type Emulator = {
    is_installed: boolean;
    binary_path: string;
    run_command: string;
};

export const useEmulatorStore = defineStore("emulatorStore", () => {
    const loading = ref(false);
    const emulators = ref<Record<string, Emulator>>({});

    async function fetchEmulators() {
        loading.value = true;
        try {
            const store = await getStore();
            const ems = await store.get<Record<string, Emulator>>("emulators");

            emulators.value = ems || {};
        } finally {
            loading.value = false;
        }
    }

    async function isEmulatorInstalled(cs: string): Promise<boolean> {
        if (Object.keys(emulators.value).length === 0) await fetchEmulators();
        return emulators.value[cs]?.is_installed || false;
    }

    return { emulators, loading, fetchEmulators, isEmulatorInstalled };
});
