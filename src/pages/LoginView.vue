<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getDomainStore, addSavedDomain, normalizeDomain, getSavedDomains, getGlobalStore } from "../lib/store";
import { onMounted, ref } from "vue";
import { useAuthStore } from "../stores/AuthStore";
import { router } from "../router";

import Button from "../components/ui/Button.vue";
import Input from "../components/ui/Input.vue";

const serverUrl = ref("");
const errorMessage = ref("");
const authStore = useAuthStore();
const isConnecting = ref(false);

const savedDomains = ref<{ domain: string, hasToken: boolean }[]>([]);

onMounted(async () => {
    const domains = await getSavedDomains();
    const domainStatuses = await Promise.all(domains.map(async (d) => {
        const store = await getDomainStore(d);
        const token = await store.get<string>("token");
        return { domain: d, hasToken: !!token };
    }));
    
    savedDomains.value = domainStatuses;
    
    if (authStore.domain && authStore.token) {
        router.push("/");
    } else if (authStore.domain) {
        serverUrl.value = authStore.domain;
    }
});

const handleSelectDomain = async (domain: string) => {
    const globalStore = await getGlobalStore();
    await globalStore.set("domain", domain);
    await globalStore.save();
    window.location.reload();
};

const handleConnect = async () => {
    if (!serverUrl.value) return;
    errorMessage.value = "";
    isConnecting.value = true;

    try {
        const normalized = normalizeDomain(serverUrl.value);
        await addSavedDomain(normalized);
        const store = await getDomainStore(normalized);
        const resp = await invoke<{ login_url: string; storage_path: string }>("get_client_start", {
            serverUrl: normalized,
        });

        await store.set("storage_path", resp.storage_path);
        await store.save();
        authStore.storagePath = resp.storage_path;

        const fullUrl = new URL(resp.login_url, normalized).href;
        await openUrl(fullUrl);
    } catch (err) {
        errorMessage.value = String(err);
    } finally {
        isConnecting.value = false;
    }
};
</script>

<template>
    <div class="login-page">
        <div class="login-card">
            <div class="header-accent"></div>

            <div class="content">
                <h1>emuNEX Login</h1>
                <p class="description">Select a server or enter a new address.</p>

                <div v-if="savedDomains.some(d => d.hasToken)" class="saved-sessions">
                    <p class="section-hint">Previous Sessions:</p>
                    <div class="domains-grid">
                        <button 
                            v-for="item in savedDomains.filter(d => d.hasToken)" 
                            :key="item.domain"
                            type="button"
                            class="session-card"
                            @click="handleSelectDomain(item.domain)"
                        >
                            <span class="session-name">{{ item.domain }}</span>
                        </button>
                    </div>
                </div>

                <div v-if="errorMessage" class="error-bubble">
                    <span class="error-icon">!</span>
                    {{ errorMessage }}
                </div>

                <form @submit.prevent="handleConnect" class="login-form">
                    <div class="add-new-title">Connect to New Server</div>
                    <Input v-model="serverUrl" label="Server Address" placeholder="https://example.com" />

                    <div class="form-actions">
                        <Button type="submit" color="blue" :disabled="isConnecting || !serverUrl">
                            {{ isConnecting ? "Connecting..." : "Connect" }}
                        </Button>
                    </div>
                </form>
            </div>

            <div class="footer-hint">Using this client requires a constant internet connection.</div>
        </div>
    </div>
</template>

<style scoped>
.login-page {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 80vh;
}

.login-card {
    width: 100%;
    max-width: 400px;
    background: #fff;
    border-radius: 15px;
    overflow: hidden;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
    border: 1px solid #ddd;
}

.header-accent {
    height: 6px;
    background: linear-gradient(90deg, #e60012 25%, #0089cf 25%, #0089cf 50%, #82d84a 50%, #82d84a 75%, #ffcc00 75%);
}

.content {
    padding: 30px;
}

h1 {
    margin: 0 0 10px 0;
    font-size: 1.5rem;
    color: #333;
    text-align: center;
}

.description {
    color: #777;
    text-align: center;
    font-size: 0.9rem;
    margin-bottom: 25px;
    line-height: 1.4;
}

.error-bubble {
    background: #fff0f0;
    border: 1px solid #ffcccc;
    color: #cc0000;
    padding: 10px;
    border-radius: 8px;
    margin-bottom: 20px;
    font-size: 0.85rem;
    display: flex;
    align-items: center;
    gap: 10px;
}

.error-icon {
    background: #cc0000;
    color: white;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
}

.saved-sessions {
    margin-bottom: 30px;
}

.section-hint {
    font-size: 0.75rem;
    color: #999;
    font-weight: 800;
    text-transform: uppercase;
    margin-bottom: 12px;
    letter-spacing: 0.5px;
}

.domains-grid {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.session-card {
    background: #fff;
    border: 2px solid #eee;
    padding: 15px;
    border-radius: 12px;
    cursor: pointer;
    text-align: left;
    transition: all 0.2s;
    display: flex;
    flex-direction: column;
    gap: 8px;
    position: relative;
    overflow: hidden;
}

.session-card:hover {
    border-color: var(--3ds-blue);
    background: #fdfdfd;
    transform: translateY(-2px);
}

.session-name {
    font-weight: 700;
    color: #444;
    font-size: 0.9rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.add-new-title {
    font-size: 0.75rem;
    color: #bbb;
    font-weight: 800;
    text-transform: uppercase;
    margin-bottom: 20px;
    display: flex;
    align-items: center;
    gap: 10px;
}

.add-new-title::before,
.add-new-title::after {
    content: "";
    flex: 1;
    height: 1px;
    background: #eee;
}

.form-actions {
    display: flex;
    justify-content: center;
    margin-top: 15px;
}

.footer-hint {
    background: #f9f9f9;
    padding: 15px;
    border-top: 1px solid #eee;
    text-align: center;
    font-size: 0.75rem;
    color: #aaa;
}
</style>
