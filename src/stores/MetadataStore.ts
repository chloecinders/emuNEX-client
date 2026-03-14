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
    const loading = ref(false);

    async function fetchCategories() {
        loading.value = true;
        try {
            const res = await http.get<Category[]>("/roms/categories");
            if (res.success) categories.value = res.data;
        } catch (err) {
            console.error("Failed to fetch categories:", err);
        } finally {
            loading.value = false;
        }
    }

    async function fetchConsoles() {
        loading.value = true;
        try {
            const res = await http.get<Console[]>("/roms/consoles");
            if (res.success) consoles.value = res.data;
        } catch (err) {
            console.error("Failed to fetch consoles:", err);
        } finally {
            loading.value = false;
        }
    }

    return {
        categories,
        consoles,
        loading,
        fetchCategories,
        fetchConsoles,
    };
});
