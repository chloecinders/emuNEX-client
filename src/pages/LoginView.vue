<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { load } from "@tauri-apps/plugin-store";
import { ref } from "vue";
import { useAuthStore } from "../stores/AuthStore";

// Components
import Button from "../components/ui/Button.vue";
import Input from "../components/ui/Input.vue";

const serverUrl = ref("");
const errorMessage = ref("");
const authStore = useAuthStore();
const isConnecting = ref(false);

const handleConnect = async () => {
    if (!serverUrl.value) return;
    errorMessage.value = "";
    isConnecting.value = true;

    try {
        const store = await load("store.json");
        const resp = await invoke<{ login_url: string; storage_path: string }>("get_client_start", {
            serverUrl: serverUrl.value,
        });

        store.set("storage_path", resp.storage_path);
        authStore.storagePath = resp.storage_path;

        const fullUrl = new URL(resp.login_url, serverUrl.value).href;
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
                <p class="description">Login to an emuNEX server.</p>

                <div v-if="errorMessage" class="error-bubble">
                    <span class="error-icon">!</span>
                    {{ errorMessage }}
                </div>

                <form @submit.prevent="handleConnect" class="login-form">
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

.form-actions {
    display: flex;
    justify-content: center;
    margin-top: 25px;
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
