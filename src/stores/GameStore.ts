import { defineStore } from "pinia";
import { ref } from "vue";
import { DiscordRPC } from "../lib/rpc";
import { http } from "../lib/http";

export type PartialGame = {
    id: string;
    title: string;
    realname: string | null;
    console: string | undefined;
    image_path: string;
    region: string;
    versions_count: number;
};

export type LibraryGame = PartialGame & {
    rom_id?: string;
    play_count: number;
    last_played: string | null;
};

export type Shelf = {
    id: string;
    name: string;
    sort_order: number;
    games: PartialGame[];
};

export type Game = {
    id: string;
    title: string;
    realname: string;
    console: string;
    region: string | null;
    category: string;
    rom_path: string;
    image_path: string;
    file_size_bytes?: number;
    release_year: number;
    languages?: string[] | null;
    versions_count: number;
};

interface FetchFilters {
    category?: string | null;
    console?: string | null;
}

export type SearchOverviewGroup = { title: string; games: PartialGame[] };

export const useGameStore = defineStore("gameStore", () => {
    const currentSelectedGame = ref<string | null>(null);
    const partialGames = ref<PartialGame[]>([]);
    const library = ref<LibraryGame[]>([]);
    const shelves = ref<Shelf[]>([]);
    const games = ref<Game[]>([]);
    const loading = ref(false);
    const isLaunching = ref(false);
    const isPlaying = ref(false);
    const isDimmed = ref(false);
    const installedGameIds = ref<string[]>([]);

    import("@tauri-apps/api/event").then(({ listen }) => {
        listen("game-status", (event) => {
            if (event.payload === "Playing") {
                isPlaying.value = true;
                isLaunching.value = false;
                isDimmed.value = true;
            } else if (event.payload === "Stopped") {
                isPlaying.value = false;
                isLaunching.value = false;
                isDimmed.value = false;
                DiscordRPC.reset();
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
                await fetchInstalledGames();
            }
        } catch (err) {
            console.error("Failed to fetch library:", err);
        } finally {
            loading.value = false;
        }
    }

    async function fetchInstalledGames() {
        try {
            const { invoke } = await import("@tauri-apps/api/core");
            const storage = await invoke<any[]>("get_local_storage");
            installedGameIds.value = storage.filter((s) => s.rom_size > 0).map((s) => s.game_id);
        } catch (err) {
            console.error("Failed to fetch installed games:", err);
        }
    }

    async function startGame(id: string) {
        try {
            const res = await http.post(`/roms/${id}/start`, {});
            if (res.success) {
                await fetchLibrary();
            } else {
                console.warn(`[GameStore] Server returned success:false for game start:`, res);
            }
        } catch (err) {
            console.error(`[GameStore] Failed to log game start for ${id}:`, err);
        }
    }

    async function fetchGame(id: string, force = false): Promise<Game | null> {
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

    async function fetchVersions(id: string): Promise<PartialGame[]> {
        loading.value = true;
        try {
            const res = await http.get<PartialGame[]>(`/roms/${id}/versions`);
            if (res.success) return res.data;
        } catch (err) {
            console.error("Failed to fetch versions:", err);
        } finally {
            loading.value = false;
        }
        return [];
    }

    const cachedSearchOverview = ref<SearchOverviewGroup[] | null>(null);

    async function fetchSearchOverview(force = false): Promise<SearchOverviewGroup[]> {
        if (!force && cachedSearchOverview.value) return cachedSearchOverview.value;

        loading.value = true;
        try {
            const res = await http.get<SearchOverviewGroup[]>("/search/overview");
            if (res.success) {
                cachedSearchOverview.value = res.data;
                return res.data;
            }
        } catch (err) {
            console.error("Failed to fetch search overview:", err);
        } finally {
            loading.value = false;
        }
        return [];
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

    async function updateShelf(id: string, data: { name?: string; sort_order?: number }) {
        try {
            const res = await http.put(`/library/shelves/${id}`, data);
            if (res.success) await fetchShelves();
        } catch (err) {
            console.error("Failed to update shelf:", err);
        }
    }

    async function deleteShelf(id: string) {
        try {
            const res = await http.delete(`/library/shelves/${id}`);
            if (res.success) await fetchShelves();
        } catch (err) {
            console.error("Failed to delete shelf:", err);
        }
    }

    async function addRomToShelf(shelfId: string, romId: string) {
        try {
            const res = await http.post(`/library/shelves/${shelfId}/roms/${romId}`, {});
            if (res.success) await fetchShelves();
        } catch (err) {
            console.error("Failed to add ROM to shelf:", err);
        }
    }

    async function removeRomFromShelf(shelfId: string, romId: string) {
        try {
            const res = await http.delete(`/library/shelves/${shelfId}/roms/${romId}`);
            if (res.success) await fetchShelves();
        } catch (err) {
            console.error("Failed to remove ROM from shelf:", err);
        }
    }

    async function updateRomOrder(shelfId: string, romIds: string[]) {
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
        fetchVersions,
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
        installedGameIds,
        fetchInstalledGames,
    };
});
