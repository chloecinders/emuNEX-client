import { defineStore } from "pinia";
import { inject, Ref, ref } from "vue";

export const useAuthStore = defineStore("authStore", () => {
    let domain: Ref<string | null> = ref(null);
    let token: Ref<string | null> = ref(null);
    let storagePath: Ref<string | null> = ref(null);

    const appProvidedDomain = inject("auth_domain");
    const appProvidedToken = inject("auth_token");
    const appProvidedStoragePath = inject("auth_storage");

    if (appProvidedDomain) {
        domain = ref(appProvidedDomain as string | null);
    }

    if (appProvidedToken) {
        token = ref(appProvidedToken as string | null);
    }

    if (appProvidedStoragePath) {
        storagePath = ref(appProvidedStoragePath as string | null);
    }

    return { domain, token, storagePath }
})
