import { defineStore } from "pinia";
import { ref } from "vue";
import { http } from "../lib/http";

export type User = {
    id: string;
    username: string;
    role: string;
};

export const useUserStore = defineStore("userStore", () => {
    const user = ref<User | null>(null);
    const loading = ref(false);

    async function fetchUser() {
        loading.value = true;

        try {
            const res = await http.get<User>("/users/@me");

            if (res.success) {
                user.value = res.data;
            }
        } catch (err) {
            user.value = null;
            console.error("Failed to fetch user:", err);
        } finally {
            loading.value = false;
        }
    }

    return { user, loading, fetchUser };
});
