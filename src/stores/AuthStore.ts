import { defineStore } from "pinia";
import { inject, Ref, ref } from "vue";

export const useAuthStore = defineStore("authStore", () => {
    const domain: Ref<string | null> = ref(null);
    const token: Ref<string | null> = ref(null);
    const storagePath: Ref<string | null> = ref(null);

    const appProvidedDomain = inject("auth_domain");
    const appProvidedToken = inject("auth_token");
    const appProvidedStoragePath = inject("auth_storage");

    if (appProvidedDomain) {
        domain.value = appProvidedDomain as string;
    }

    if (appProvidedToken) {
        token.value = appProvidedToken as string;
    }

    if (appProvidedStoragePath) {
        storagePath.value = appProvidedStoragePath as string;
    }

    return { domain, token, storagePath }
})
