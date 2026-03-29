import { defineStore } from "pinia";
import { ref } from "vue";
import { http } from "../lib/http";

export type ConsoleMetadata = {
    name: string;
    card_color: string | null;
};

export const useConsoleStore = defineStore("consoleStore", () => {
    const consoles = ref<Record<string, ConsoleMetadata>>({});
    const loading = ref(false);

    async function fetchConsoles() {
        if (Object.keys(consoles.value).length > 0) return;

        loading.value = true;
        try {
            const res = await http.get<ConsoleMetadata[]>("/consoles");
            if (res.success) {
                const map: Record<string, ConsoleMetadata> = {};
                for (const c of res.data) {
                    map[c.name] = c;
                }
                consoles.value = map;
            }
        } catch (err) {
            console.error("Failed to fetch consoles:", err);
        } finally {
            loading.value = false;
        }
    }

    function getConsoleColor(consoleName: string | undefined): string {
        if (!consoleName) return "var(--color-surface)";
        const metadata = consoles.value[consoleName];
        return metadata?.card_color || "var(--color-surface)";
    }

    return {
        consoles,
        loading,
        fetchConsoles,
        getConsoleColor,
    };
});
