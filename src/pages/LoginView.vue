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

const savedDomains = ref<{ domain: string; hasToken: boolean }[]>([]);

onMounted(async () => {
    const domains = await getSavedDomains();
    const domainStatuses = await Promise.all(
        domains.map(async (d) => {
            const store = await getDomainStore(d);
            const token = await store.get<string>("token");
            return { domain: d, hasToken: !!token };
        }),
    );

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

        await store.set("domain", normalized);
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
    <div class="c-login">
        <div class="c-login__card">
            <header class="c-login__header">
                <h1 class="c-login__title">emuNEX</h1>
                <p class="c-login__tagline">Remote Emulation & Library Management</p>
            </header>

            <div class="c-login__content">
                <div v-if="savedDomains.some(d => d.hasToken)" class="c-login__sessions">
                    <p class="c-login__section-hint">Select a Library</p>

                    <div class="c-login__domains-grid">
                        <button 
                            v-for="item in savedDomains.filter(d => d.hasToken)" 
                            :key="item.domain"
                            type="button"
                            class="c-login__session-card"
                            @click="handleSelectDomain(item.domain)"
                        >
                            <span class="c-login__session-name">{{ item.domain }}</span>

                            <span class="c-login__session-arrow">→</span>
                        </button>
                    </div>
                </div>

                <div v-if="errorMessage" class="c-login__error-bubble">
                    {{ errorMessage }}
                </div>

                <form @submit.prevent="handleConnect" class="c-login__form">
                    <div class="c-login__add-new-title">Connect to Server</div>
                    <Input 
                        v-model="serverUrl" 
                        label="Server Address" 
                        placeholder="https://emunex.example.com"
                    />
                    
                    <div class="c-login__actions">
                        <Button type="submit" color="blue" :disabled="isConnecting || !serverUrl">
                            {{ isConnecting ? "CONNECTING..." : "CONNECT" }}
                        </Button>
                    </div>
                </form>
            </div>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-login {
    min-height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-xl);

    &__card {
        background: var(--color-surface);
        width: 100%;
        max-width: 440px;
        border-radius: var(--radius-md);
        box-shadow: var(--shadow-subtle);
        border: 1px solid var(--color-border);
        overflow: hidden;
        animation: card-appear 0.4s ease-out;
    }

    &__header {
        padding: var(--spacing-lg) var(--spacing-lg) var(--spacing-md) var(--spacing-lg);
        text-align: center;
    }

    &__title {
        font-size: 2.5rem;
        font-weight: 900;
        margin: 0;
        letter-spacing: -1.5px;
        color: var(--color-primary);
    }

    &__tagline {
        margin-top: var(--spacing-xs);
        font-size: 0.75rem;
        font-weight: 800;
        color: var(--color-text-muted);
        text-transform: uppercase;
        letter-spacing: 2px;
    }

    &__content {
        padding: var(--spacing-lg);
        display: flex;
        flex-direction: column;
        gap: var(--spacing-md);
    }

    &__section-hint, &__add-new-title {
        font-size: 0.7rem;
        font-weight: 900;
        color: var(--color-text-muted);
        text-transform: uppercase;
        letter-spacing: 1px;
        margin-bottom: var(--spacing-md);
        display: flex;
        align-items: center;
        gap: var(--spacing-md);

        &::after {
            content: "";
            flex: 1;
            height: 1px;
            background: var(--color-border);
        }
    }

    &__domains-grid {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }

    &__session-card {
        background: var(--color-surface-variant);
        border: 2px solid var(--color-border);
        padding: var(--spacing-md) var(--spacing-lg);
        border-radius: var(--radius-md);
        cursor: pointer;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;

        &:hover {
            border-color: var(--color-primary);
            background: var(--color-surface);
            transform: translateY(-var(--spacing-xxs));
            box-shadow: var(--shadow-subtle);

            .c-login__session-arrow {
                opacity: 1;
            }
        }
    }

    &__session-name {
        font-weight: 800;
        color: var(--color-text);
        font-size: 0.95rem;
    }

    &__session-arrow {
        color: var(--color-primary);
        font-weight: 900;
        font-size: 1.1rem;
        opacity: 0.4;
    }

    &__form {
        display: flex;
        flex-direction: column;
    }

    &__actions {
        margin-top: var(--spacing-md);

        :deep(.c-button) {
            width: 100% !important;
            display: block;
        }
    }

    &__error-bubble {
        background: #fff5f5;
        border: 1px solid #ffccd1;
        padding: var(--spacing-md) var(--spacing-lg);
        border-radius: var(--radius-md);
        color: #e53e3e;
        font-weight: 700;
        font-size: 0.85rem;
        text-align: center;
    }
}

@keyframes card-appear {
    from { opacity: 0; transform: translateY(var(--spacing-sm)); }
    to { opacity: 1; transform: translateY(0); }
}
</style>
