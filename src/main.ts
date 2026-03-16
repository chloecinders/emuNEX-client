import { onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { createPinia } from "pinia";
import { createApp } from "vue";
import App from "./App.vue";
import { addSavedDomain, getDomainStore, getGlobalStore, normalizeDomain } from "./lib/store";
import { router } from "./router";

async function checkForUpdates() {
    try {
        const update = await check();
        if (update) {
            await update.downloadAndInstall();
            await relaunch();
        }
    } catch (e) {
        console.warn("Couldn't check/download updates:", e);
    }
}

await checkForUpdates();

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);

import { useAuthStore } from "./stores/AuthStore";

const authStore = useAuthStore(pinia);

app.use(router);

app.mount("#app");

const globalStore = await getGlobalStore();

const urlHandler = async (urls: string[]) => {
    const url = new URL(urls[0]);

    if (url.host === "login") {
        const domain = url.searchParams.get("domain");
        const token = url.searchParams.get("token");
        const storagePath = url.searchParams.get("storage_path");

        if (domain) {
            const normalizedDomain = normalizeDomain(domain);
            await addSavedDomain(normalizedDomain);
            const domainStore = await getDomainStore(normalizedDomain);
            await domainStore.set("domain", normalizedDomain);
            await domainStore.set("token", token);

            if (storagePath) {
                await domainStore.set("storage_path", storagePath);
            }

            await domainStore.save();
            await globalStore.set("domain", normalizedDomain);
            await globalStore.save();

            authStore.setAuth(normalizedDomain, token || "", storagePath || "");
        }

        window.location.href = "/";
    }
};

onOpenUrl(urlHandler);

const domain = await globalStore.get<string>("domain");

if (domain) {
    const domainStore = await getDomainStore(domain);
    const token = await domainStore.get<string>("token");
    const storagePath = await domainStore.get<string>("storage_path");

    if (token && storagePath) {
        authStore.setAuth(domain, token, storagePath);
        authStore.startup().then(() => {
            if (router.currentRoute.value.name === "login") {
                router.push("/");
            }
        });
    }
}
