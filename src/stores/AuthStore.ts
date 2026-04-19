import { defineStore } from "pinia";
import { ref } from "vue";

export const useAuthStore = defineStore("authStore", () => {
    const domain = ref<string | null>(null);
    const token = ref<string | null>(null);
    const storagePath = ref<string | null>(null);

    const isConnecting = ref(false);
    const connectionError = ref<string | null>(null);

    function setAuth(newDomain: string, newToken: string, newStoragePath: string) {
        domain.value = newDomain;
        token.value = newToken;
        storagePath.value = newStoragePath;
        connectionError.value = null;
    }

    function clearAuth() {
        domain.value = null;
        token.value = null;
        storagePath.value = null;
        connectionError.value = null;
    }

    async function initConnection() {
        if (!domain.value) {
            return;
        }

        isConnecting.value = true;
        connectionError.value = null;

        try {
            const { http } = await import("../lib/http");
            const res = await http.get("/start");
            if (res.success) {
                return true;
            }
        } catch (err: any) {
            connectionError.value = err.error || "Failed to reach server.";
        } finally {
            isConnecting.value = false;
        }

        return false;
    }

    async function logout() {
        clearAuth();
        const { getGlobalStore } = await import("../lib/store");
        const globalStore = await getGlobalStore();
        await globalStore.set("domain", null);
        await globalStore.save();
        window.location.reload();
    }

    async function invalidateSession() {
        const currentDomain = domain.value;
        clearAuth();
        
        const { getGlobalStore, getDomainStore } = await import("../lib/store");
        const globalStore = await getGlobalStore();
        await globalStore.set("domain", null);
        await globalStore.save();

        if (currentDomain) {
            const domainStore = await getDomainStore(currentDomain);
            await domainStore.set("token", null);
            await domainStore.save();
        }

        window.location.reload();
    }

    async function startup() {
        if (!domain.value) {
            return;
        }

        const connected = await initConnection();
        if (connected) {
            const { useUserStore } = await import("./UserStore");
            const { useGameStore } = await import("./GameStore");
            const { useMetadataStore } = await import("./MetadataStore");

            const userStore = useUserStore();
            const gameStore = useGameStore();
            const metadataStore = useMetadataStore();

            await Promise.all([
                userStore.fetchUser(),
                gameStore.fetchPartialGames(),
                gameStore.fetchLibrary(),
                metadataStore.fetchCategories(),
                metadataStore.fetchConsoles(),
            ]);
        }
    }

    return {
        domain,
        token,
        storagePath,
        isConnecting,
        connectionError,
        setAuth,
        clearAuth,
        initConnection,
        logout,
        invalidateSession,
        startup,
    };
});
