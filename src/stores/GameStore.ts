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

export type Shelf = {
    id: number;
    name: string;
    sort_order: number;
    games: PartialGame[];
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
    const shelves = ref<Shelf[]>([]);
    const games = ref<Game[]>([]);
    const loading = ref(false);
    const isLaunching = ref(false);
    const isPlaying = ref(false);
    const isDimmed = ref(false);

    import("@tauri-apps/api/event").then(({ listen }) => {
        listen("game-status", (event) => {
            if (event.payload === "Playing") {
                isPlaying.value = true;
                isLaunching.value = false;
                isDimmed.value = true;
            } else if (event.payload === "Stopped") {
                isPlaying.value = false;
                isLaunching.value = false;
                // We don't automatically set isDimmed to false here 
                // because the user might have already dismissed it or 
                // we might want it to stay until cleanup is done.
                // But for a fresh "Stopped" event, let's reset it if it was still active.
                isDimmed.value = false;
            }
        });
    });

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
            const foundGame = games.value.find((g) => g.id === id);
            if (foundGame) return foundGame;
        }

        try {
            const res = await http.get<Game>(`/roms/${id}`);

            if (res.success) {
                games.value = games.value.filter((g) => g.id !== id);
                games.value.push(res.data);
                return res.data;
            }
        } catch (err) {
            console.error("Failed to fetch game details:", err);
        }

        return null;
    }

    async function searchGames(query: string, filters: FetchFilters = {}): Promise<PartialGame[]> {
        if (!query.trim() && !filters.category && !filters.console) return [];
        loading.value = true;

        const params = new URLSearchParams();
        params.append("query", query);
        if (filters.category) params.append("category", filters.category);
        if (filters.console) params.append("console", filters.console);

        try {
            const res = await http.get<PartialGame[]>(`/roms/search?${params.toString()}`);
            if (res.success) return res.data;
        } catch (err) {
            console.error("Search failed:", err);
        } finally {
            loading.value = false;
        }

        return [];
    }

    const cachedSearchOverview = ref<Record<string, PartialGame[]> | null>(null);

    async function fetchSearchOverview(force = false): Promise<Record<string, PartialGame[]>> {
        if (!force && cachedSearchOverview.value) return cachedSearchOverview.value;

        loading.value = true;
        try {
            const res = await http.get<Record<string, PartialGame[]>>("/search/overview");
            if (res.success) {
                cachedSearchOverview.value = res.data;
                return res.data;
            }
        } catch (err) {
            console.error("Failed to fetch search overview:", err);
        } finally {
            loading.value = false;
        }
        return {};
    }

    async function fetchShelves() {
        loading.value = true;
        try {
            const res = await http.get<Shelf[]>("/library/shelves");
            if (res.success) shelves.value = res.data;
        } catch (err) {
            console.error("Failed to fetch shelves:", err);
        } finally {
            loading.value = false;
        }
    }

    async function createShelf(name: string) {
        try {
            const res = await http.post<number>("/library/shelves", { name });
            if (res.success) {
                await fetchShelves();
                return res.data;
            }
        } catch (err) {
            console.error("Failed to create shelf:", err);
        }
        return null;
    }

    async function updateShelf(id: number, data: { name?: string; sort_order?: number }) {
        try {
            const res = await http.put(`/library/shelves/${id}`, data);
            if (res.success) await fetchShelves();
        } catch (err) {
            console.error("Failed to update shelf:", err);
        }
    }

    async function deleteShelf(id: number) {
        try {
            const res = await http.delete(`/library/shelves/${id}`);
            if (res.success) await fetchShelves();
        } catch (err) {
            console.error("Failed to delete shelf:", err);
        }
    }

    async function addRomToShelf(shelfId: number, romId: number) {
        try {
            const res = await http.post(`/library/shelves/${shelfId}/roms/${romId}`, {});
            if (res.success) await fetchShelves();
        } catch (err) {
            console.error("Failed to add ROM to shelf:", err);
        }
    }

    async function removeRomFromShelf(shelfId: number, romId: number) {
        try {
            const res = await http.delete(`/library/shelves/${shelfId}/roms/${romId}`);
            if (res.success) await fetchShelves();
        } catch (err) {
            console.error("Failed to remove ROM from shelf:", err);
        }
    }

    async function updateRomOrder(shelfId: number, romIds: number[]) {
        try {
            const res = await http.put(`/library/shelves/${shelfId}/roms/order`, { rom_ids: romIds });
            if (res.success) await fetchShelves();
        } catch (err) {
            console.error("Failed to update ROM order:", err);
        }
    }

    return {
        currentSelectedGame,
        partialGames,
        library,
        shelves,
        games,
        loading,
        fetchPartialGames,
        fetchLibrary,
        startGame,
        fetchGame,
        searchGames,
        fetchSearchOverview,
        fetchShelves,
        createShelf,
        updateShelf,
        deleteShelf,
        addRomToShelf,
        removeRomFromShelf,
        updateRomOrder,
        isLaunching,
        isPlaying,
        isDimmed,
    };
});
