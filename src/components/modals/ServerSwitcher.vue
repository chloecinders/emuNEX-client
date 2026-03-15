<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { onMounted, ref, watch } from "vue";
import { getDomainStore, getGlobalStore, getSavedDomains, normalizeDomain } from "../../lib/store";
import Button from "../ui/Button.vue";
import Input from "../ui/Input.vue";

const props = defineProps<{
    show: boolean;
}>();

const emit = defineEmits(["close", "switch"]);

const domains = ref<string[]>([]);
const newDomain = ref("");
const errorMessage = ref("");

const isConnecting = ref(false);

async function loadDomains() {
    const rawDomains = await getSavedDomains();

    const domainStatuses = await Promise.all(
        rawDomains.map(async (d) => {
            const store = await getDomainStore(d);
            const token = await store.get<string>("token");
            return { domain: d, hasToken: !!token };
        }),
    );

    domains.value = domainStatuses.filter((d) => d.hasToken).map((d) => d.domain);
}

async function startConnection(normalizedDomain: string) {
    isConnecting.value = true;
    errorMessage.value = "";
    try {
        const store = await getDomainStore(normalizedDomain);
        const resp = await invoke<{ login_url: string; storage_path: string }>("get_client_start", {
            serverUrl: normalizedDomain,
        });

        await store.set("storage_path", resp.storage_path);
        await store.save();

        const fullUrl = new URL(resp.login_url, normalizedDomain).href;
        await openUrl(fullUrl);
        emit("close");
    } catch (err) {
        errorMessage.value = String(err);
    } finally {
        isConnecting.value = false;
    }
}

async function handleSwitch(domain: string) {
    const normalized = normalizeDomain(domain);
    const store = await getDomainStore(normalized);
    const token = await store.get<string>("token");

    if (token) {
        const globalStore = await getGlobalStore();
        await globalStore.set("domain", normalized);
        await globalStore.save();
        window.location.reload();
    } else {
        await startConnection(normalized);
    }
}

async function handleAdd() {
    const rawDomain = newDomain.value.trim();
    if (!rawDomain) return;

    const normalized = normalizeDomain(rawDomain);

    try {
        new URL(normalized);
    } catch (e) {
        errorMessage.value = "Invalid server address";
        return;
    }

    await startConnection(normalized);
}

watch(
    () => props.show,
    (newVal) => {
        if (newVal) loadDomains();
    },
);

onMounted(loadDomains);
</script>

<template>
    <transition name="fade">
        <div v-if="show" class="modal-overlay" @click.self="emit('close')">
            <div class="modal-card">
                <button class="modal-close" @click="emit('close')">
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                        <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                    </svg>
                </button>

                <div class="modal-header">
                    <h2 class="title">Switch Server</h2>
                    <p class="subtitle">Select a previously connected server or add a new one.</p>
                </div>

                <div class="modal-body">
                    <div v-if="domains.length > 0" class="domain-list">
                        <div v-for="domain in domains" :key="domain" class="domain-item" @click="handleSwitch(domain)">
                            <div class="domain-info">
                                <span class="domain-name">{{ domain }}</span>
                            </div>
                            <span class="chevron">&rsaquo;</span>
                        </div>
                    </div>

                    <div class="add-section">
                        <div class="section-label">ADD NEW SERVER</div>
                        <div class="add-form">
                            <Input v-model="newDomain" placeholder="https://emunex.example.com" />

                            <div class="btn-wrap">
                                <Button @click="handleAdd" color="blue" :disabled="isConnecting">
                                    {{ isConnecting ? "..." : "Connect" }}
                                </Button>
                            </div>
                        </div>

                        <p v-if="errorMessage" class="error-text">{{ errorMessage }}</p>
                    </div>
                </div>
            </div>
        </div>
    </transition>
</template>

<style scoped>
.modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(10, 10, 30, 0.4);
    backdrop-filter: blur(var(--spacing-sm));
    -webkit-backdrop-filter: blur(var(--spacing-sm));
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
}

.modal-card {
    background: var(--color-surface);
    border-radius: var(--radius-md);
    width: 90%;
    max-width: 440px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.12);
    border: 1px solid var(--color-border);
    position: relative;
    padding: var(--spacing-xl);
    animation: modal-pop 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes modal-pop {
    from {
        transform: scale(0.95);
        opacity: 0;
    }
    to {
        transform: scale(1);
        opacity: 1;
    }
}

.modal-close {
    position: absolute;
    top: var(--spacing-md);
    right: var(--spacing-md);
    width: var(--spacing-xl);
    height: var(--spacing-xl);
    border-radius: var(--radius-full);
    border: none;
    background: var(--color-surface-variant);
    color: var(--color-text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    z-index: 10;
}

.modal-close:hover {
    background: var(--color-primary);
    color: white;
}

.modal-header {
    text-align: center;
    margin-bottom: var(--spacing-xl);
}

.title {
    font-size: 1.5rem;
    font-weight: 950;
    color: var(--color-primary);
    margin: 0;
    letter-spacing: -0.5px;
}

.subtitle {
    margin-top: var(--spacing-sm);
    font-size: 0.85rem;
    color: var(--color-text-muted);
    font-weight: 700;
}

.domain-list {
    margin-bottom: var(--spacing-xl);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
}

.domain-item {
    background: var(--color-surface-variant);
    padding: var(--spacing-md) var(--spacing-lg);
    border-radius: var(--radius-md);
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    transition: all 0.2s;
    border: var(--spacing-xxs) solid var(--color-border);
}

.domain-item:hover {
    border-color: var(--color-primary);
    background: var(--color-surface);
    transform: translateY(-var(--spacing-xxs));
    box-shadow: var(--shadow-subtle);
}

.domain-info {
    display: flex;
    align-items: center;
}

.domain-name {
    font-weight: 800;
    font-size: 0.95rem;
    color: var(--color-text);
}

.chevron {
    color: var(--color-primary);
    font-weight: 900;
    font-size: 1.2rem;
    opacity: 0.4;
    line-height: 1;
    display: flex;
    align-items: center;
}

.domain-item:hover .chevron {
    opacity: 1;
}

.section-label {
    font-size: 0.7rem;
    color: var(--color-primary);
    font-weight: 900;
    letter-spacing: 1px;
    margin-bottom: var(--spacing-sm);
}

.add-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
}

.btn-wrap {
    width: 100%;
}

.btn-wrap :deep(.nintendo-btn) {
    width: 100%;
}

.error-text {
    color: #ff4d4f;
    font-size: 0.8rem;
    margin-top: var(--spacing-sm);
    font-weight: 700;
    text-align: center;
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.3s ease;
}
.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
