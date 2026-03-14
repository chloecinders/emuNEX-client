import { onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { relaunch } from '@tauri-apps/plugin-process';
import { load } from "@tauri-apps/plugin-store";
import { check } from '@tauri-apps/plugin-updater';
import { createPinia } from "pinia";
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";

async function checkForUpdates() {
    const update = await check();
    if (update) {
        await update.downloadAndInstall();
        await relaunch();
    }
}

await checkForUpdates();

const store = await load("store.json");

const app = createApp(App).use(router)

const urlHandler = async (urls: string[]) => {
    const url = new URL(urls[0]);

    if (url.host == "login") {
        const domain = url.searchParams.get("domain");
        const token = url.searchParams.get("token");

        app.provide("auth_domain", domain);
        app.provide("auth_token", token);
        await store.set("domain", domain);
        await store.set("token", token);

        router.push("/");
        window.location.reload();
    }
}

onOpenUrl(urlHandler);

const pinia = createPinia();

const domain = await store.get<string>("domain");
const token = await store.get<string>("token");
const storagePath = await store.get<string>("storage_path");

if (domain) {
    app.provide("auth_domain", domain);
}

if (token) {
    app.provide("auth_token", token);
}

if (storagePath) {
    app.provide("auth_storage", storagePath);
}

app.use(pinia).mount("#app");
