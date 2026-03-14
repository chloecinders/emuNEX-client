<script lang="ts" setup>
import { onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getSavedDomains, getGlobalStore, normalizeDomain, getDomainStore } from "../../lib/store";
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
    const domainStatuses = await Promise.all(rawDomains.map(async (d) => {
        const store = await getDomainStore(d);
        const token = await store.get<string>("token");
        return { domain: d, hasToken: !!token };
    }));
    
    domains.value = domainStatuses.filter(d => d.hasToken).map(d => d.domain);
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

watch(() => props.show, (newVal) => {
    if (newVal) loadDomains();
});

onMounted(loadDomains);
</script>

<template>
    <transition name="fade">
        <div v-if="show" class="modal-overlay" @click.self="emit('close')">
            <div class="modal-content">
                <div class="modal-header">
                    <h3>Switch Server</h3>
                    <button class="close-btn" @click="emit('close')">&times;</button>
                </div>
                
                <div class="modal-body">
                    <div v-if="domains.length > 0" class="domain-list">
                        <div 
                            v-for="domain in domains" 
                            :key="domain" 
                            class="domain-item"
                            @click="handleSwitch(domain)"
                        >
                            <span class="domain-name">{{ domain }}</span>
                            <span class="chevron">&rsaquo;</span>
                        </div>
                    </div>
                    
                    <div class="add-section">
                        <h4>Add New Server</h4>
                        <div class="add-form">
                            <div class="input-wrapper">
                                <Input v-model="newDomain" placeholder="https://example.com" />
                            </div>
                            <Button @click="handleAdd" color="blue" :disabled="isConnecting">
                                {{ isConnecting ? "..." : "Connect" }}
                            </Button>
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
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
}

.modal-content {
    background: #fff;
    border: 3px solid var(--3ds-blue, #0089cf);
    border-radius: 15px;
    width: 90%;
    max-width: 400px;
    box-shadow: 0 15px 35px rgba(0, 0, 0, 0.4);
    overflow: hidden;
    animation: pop-in 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

@keyframes pop-in {
    from { transform: scale(0.8); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
}

.modal-header {
    background: linear-gradient(to right, var(--3ds-blue, #0089cf), #00b0ff);
    padding: 15px 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: white;
}

.modal-header h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 900;
    text-transform: uppercase;
}

.close-btn {
    background: none;
    border: none;
    color: white;
    font-size: 1.5rem;
    cursor: pointer;
    line-height: 1;
}

.modal-body {
    padding: 20px;
}

.domain-list {
    margin-bottom: 25px;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.domain-item {
    background: #f8f8f8;
    padding: 12px 16px;
    border-radius: 10px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid #eee;
}

.domain-item:hover {
    background: #f0f0f0;
    border-color: #ddd;
    transform: translateX(5px);
}

.domain-name {
    font-weight: 700;
    color: #444;
}

.chevron {
    font-size: 1.5rem;
    color: #bbb;
}

.add-section h4 {
    margin: 0 0 12px 0;
    font-size: 0.9rem;
    color: #888;
    text-transform: uppercase;
}

.add-form {
    display: flex;
    gap: 10px;
    align-items: flex-end;
}

.input-wrapper {
    flex: 1;
}

.add-form :deep(.input-container) {
    margin-bottom: 0;
}

.error-text {
    color: var(--3ds-red, #e60012);
    font-size: 0.8rem;
    margin-top: 8px;
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
