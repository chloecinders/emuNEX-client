import { defineStore } from "pinia";
import { ref } from "vue";
import { http } from "../utils/http";

export type User = {
    id: number,
    username: string,
    role: string,
};

export const useUserStore = defineStore("userStore", () => {
    const user = ref<User | null>(null);
    const loading = ref(false);

    async function fetchUser(force = false): Promise<User | null> {
        if (user.value && !force) return user.value;

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

        return user.value;
    }

    fetchUser();

    return { user, loading, fetchUser };
});
