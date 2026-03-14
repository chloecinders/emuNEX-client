import { defineStore } from "pinia";
import { ref } from "vue";

export const useAuthStore = defineStore("authStore", () => {
    const domain = ref<string | null>(null);
    const token = ref<string | null>(null);
    const storagePath = ref<string | null>(null);

    function setAuth(newDomain: string, newToken: string, newStoragePath: string) {
        domain.value = newDomain;
        token.value = newToken;
        storagePath.value = newStoragePath;
    }

    function clearAuth() {
        domain.value = null;
        token.value = null;
        storagePath.value = null;
    }

    return { domain, token, storagePath, setAuth, clearAuth }
})
