<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ChevronRight } from "lucide-vue-next";
import { onMounted, ref, watch } from "vue";
import { getDomainStore, getGlobalStore, getSavedDomains, normalizeDomain } from "../../lib/store";
import Button from "../ui/Button.vue";
import Input from "../ui/Input.vue";
import Modal from "../ui/Modal.vue";

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

        await store.set("domain", normalizedDomain);
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
        await store.set("domain", normalized);
        await store.save();

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
    <Modal
        :show="show"
        title="Switch Server"
        subtitle="Select a previously connected server or add a new one."
        @close="emit('close')"
    >
        <div v-if="domains.length > 0" class="c-server-switcher__list">
            <div v-for="domain in domains" :key="domain" class="c-server-switcher__item" @click="handleSwitch(domain)">
                <div class="c-server-switcher__info">
                    <span class="c-server-switcher__name">{{ domain }}</span>
                </div>
                <ChevronRight class="c-server-switcher__chevron" />
            </div>
        </div>

        <div class="c-server-switcher__add-section">
            <div class="c-server-switcher__section-label">ADD NEW SERVER</div>
            <div class="c-server-switcher__form">
                <Input v-model="newDomain" placeholder="https://emunex.example.com" />

                <div class="c-server-switcher__actions">
                    <Button @click="handleAdd" color="primary" :disabled="isConnecting">
                        {{ isConnecting ? "..." : "Connect" }}
                    </Button>
                </div>
            </div>

            <p v-if="errorMessage" class="c-server-switcher__error">{{ errorMessage }}</p>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-server-switcher {
    &__list {
        margin-bottom: var(--spacing-xl);
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }

    &__item {
        background: var(--color-surface-variant);
        padding: var(--spacing-md) var(--spacing-lg);
        border-radius: var(--radius-md);
        display: flex;
        justify-content: space-between;
        align-items: center;
        cursor: pointer;
        transition: all 0.2s;
        border: var(--spacing-xxs) solid var(--color-border);

        &:hover {
            border-color: var(--color-primary);
            background: var(--color-surface);
            transform: translateY(-var(--spacing-xxs));
            box-shadow: var(--shadow-subtle);

            .c-server-switcher__chevron {
                opacity: 1;
            }
        }
    }

    &__info {
        display: flex;
        align-items: center;
    }

    &__name {
        font-weight: 800;
        font-size: 0.95rem;
        color: var(--color-text);
    }

    &__chevron {
        color: var(--color-primary);
        width: 24px;
        height: 24px;
        opacity: 0.4;
        transition: all 0.2s;
    }

    &__section-label {
        font-size: 0.7rem;
        color: var(--color-primary);
        font-weight: 900;
        letter-spacing: 1px;
        margin-bottom: var(--spacing-sm);
    }

    &__form {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-md);
    }

    &__actions {
        width: 100%;

        :deep(.c-button) {
            width: 100%;
        }
    }

    &__error {
        color: #ff4d4f;
        font-size: 0.8rem;
        margin-top: var(--spacing-sm);
        font-weight: 700;
        text-align: center;
    }
}
</style>
