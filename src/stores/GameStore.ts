import { defineStore } from "pinia";
import { ref } from "vue";
import { http } from "../utils/http";

export type PartialGame = {
    id: number;
    title: string;
    console: string | undefined;
    image_path: string;
};

export type LibraryGame = PartialGame & {
    play_count: number;
    last_played: string | null;
};

export type Game = {
    id: number;
    title: string;
    console: string;
    region: string | null;
    category: string;
    rom_path: string;
    image_path: string;
};

interface FetchFilters {
    category?: string | null;
    console?: string | null;
}

export const useGameStore = defineStore("gameStore", () => {
    const currentSelectedGame = ref<number | null>(null);
    const partialGames = ref<PartialGame[]>([]);
    const library = ref<LibraryGame[]>([]);
    const games = ref<Game[]>([]);
    const loading = ref(false);

    async function fetchPartialGames(filters: FetchFilters = {}) {
        const params = new URLSearchParams();
        if (filters.category) params.append("category", filters.category);
        if (filters.console) params.append("console", filters.console);

        const queryString = params.toString();
        const url = `/roms/list${queryString ? `?${queryString}` : ""}`;

        loading.value = true;

        try {
            const res = await http.get<PartialGame[]>(url);
            if (res.success) {
                if (!queryString) partialGames.value = res.data;
                return res.data;
            }
        } catch (err) {
            console.error("Failed to fetch games:", err);
        } finally {
            loading.value = false;
        }
        return [];
    }

    async function fetchLibrary() {
        loading.value = true;
        try {
            const res = await http.get<LibraryGame[]>("/library");
            if (res.success) {
                library.value = res.data;
            }
        } catch (err) {
            console.error("Failed to fetch library:", err);
        } finally {
            loading.value = false;
        }
    }

    async function startGame(id: number) {
        try {
            await http.post(`/roms/${id}/start`, {});
            await fetchLibrary();
        } catch (err) {
            console.error("Failed to log game start:", err);
        }
    }

    async function fetchGame(id: number, force = false): Promise<Game | null> {
        if (!force) {
            const foundGame = games.value.find(g => g.id === id);
            if (foundGame) return foundGame;
        }

        try {
            const res = await http.get<Game>(`/roms/${id}`);

            if (res.success) {
                games.value = games.value.filter(g => g.id !== id);
                games.value.push(res.data);
                return res.data;
            }
        } catch (err) {
            console.error("Failed to fetch game details:", err);
        }

        return null;
    }

    async function searchGames(query: string): Promise<PartialGame[]> {
        if (!query.trim()) return [];
        loading.value = true;

        try {
            const res = await http.get<PartialGame[]>(`/roms/search?query=${encodeURIComponent(query)}`);
            if (res.success) return res.data;
        } catch (err) {
            console.error("Search failed:", err);
        } finally {
            loading.value = false;
        }

        return [];
    }

    return {
        currentSelectedGame,
        partialGames,
        library,
        games,
        loading,
        fetchPartialGames,
        fetchLibrary,
        startGame,
        fetchGame,
        searchGames
    };
});
