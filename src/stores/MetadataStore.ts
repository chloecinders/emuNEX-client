import { defineStore } from "pinia";
import { ref } from "vue";
import { http } from "../utils/http";

export interface Category {
    name: string;
}

export interface Console {
    name: string;
    card_color: string | undefined;
}

export const useMetadataStore = defineStore("metadata", () => {
    const categories = ref<Category[]>([]);
    const consoles = ref<Console[]>([]);

    const loadingCategories = ref(false);
    const loadingConsoles = ref(false);

    async function fetchCategories() {
        if (categories.value.length > 0) return;

        loadingCategories.value = true;
        try {
            const res = await http.get<Category[]>("/roms/categories");
            categories.value = res.data;
        } catch (err) {
            console.error("Failed to fetch categories:", err);
        } finally {
            loadingCategories.value = false;
        }
    }

    async function fetchConsoles() {
        if (consoles.value.length > 0) return;

        loadingConsoles.value = true;
        try {
            const res = await http.get<Console[]>("/roms/consoles");
            consoles.value = res.data;
        } catch (err) {
            console.error("Failed to fetch consoles:", err);
        } finally {
            loadingConsoles.value = false;
        }
    }

    return {
        categories,
        consoles,
        loadingCategories,
        loadingConsoles,
        fetchCategories,
        fetchConsoles
    };
});
