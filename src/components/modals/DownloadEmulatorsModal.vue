<script setup lang="ts">
import { Search } from "lucide-vue-next";
import { computed, ref } from "vue";
import { formatBytes } from "../../lib/format";
import { useConsoleStore } from "../../stores/ConsoleStore";
import type { ServerEmulator } from "../../stores/EmulatorStore";
import Button from "../ui/Button.vue";
import Heading from "../ui/Heading.vue";
import Modal from "../ui/Modal.vue";
import Spinner from "../ui/Spinner.vue";
import Text from "../ui/Text.vue";

const props = defineProps<{
    show: boolean;
    serverEmulators: ServerEmulator[];
    isFetching: boolean;
    loading: boolean;
}>();

const emit = defineEmits<{
    (e: "close"): void;
    (e: "download", serverEmulator: ServerEmulator): void;
}>();

const consoleStore = useConsoleStore();
const searchQuery = ref("");

const groupedEmulators = computed(() => {
    const query = searchQuery.value.toLowerCase().trim();

    const filtered = props.serverEmulators.filter((se) => {
        if (!query) return true;
        const matchName = se.name.toLowerCase().includes(query);
        const matchConsole = se.consoles.some((c) => c.toLowerCase().includes(query));
        return matchName || matchConsole;
    });

    const uniqueEmulatorsMap = new Map<string, ServerEmulator>();
    for (const se of filtered) {
        uniqueEmulatorsMap.set(se.id, se);
    }

    const groups: Record<string, ServerEmulator[]> = {};
    for (const se of uniqueEmulatorsMap.values()) {
        const targetConsoles = se.consoles && se.consoles.length > 0 ? se.consoles : ["OTHER"];
        for (const c of targetConsoles) {
            const consoleName = c.toUpperCase();
            if (!groups[consoleName]) {
                groups[consoleName] = [];
            }
            groups[consoleName].push(se);
        }
    }

    return Object.keys(groups)
        .sort()
        .map((consoleName) => ({
            console: consoleName,
            emulators: groups[consoleName],
        }));
});
</script>

<template>
    <Modal :show="show" title="Download Emulators" width="800px" @close="emit('close')">
        <div class="c-download-modal">
            <div class="c-download-modal__search-wrap">
                <Search class="c-download-modal__search-icon" :size="18" />
                <input
                    v-model="searchQuery"
                    class="c-download-modal__search"
                    placeholder="Search by name or console..." />
            </div>

            <div v-if="isFetching" class="c-download-modal__loading">
                <Spinner />
                <Text>Fetching available emulators...</Text>
            </div>

            <div v-else-if="serverEmulators.length === 0" class="c-download-modal__empty">
                <Text variant="muted">No emulators available for download on the server.</Text>
            </div>

            <div v-else-if="groupedEmulators.length === 0" class="c-download-modal__empty">
                <Text variant="muted">No emulators match your search.</Text>
            </div>

            <div v-else class="c-download-modal__content">
                <div v-for="group in groupedEmulators" :key="group.console" class="c-download-group">
                    <div class="c-download-group__header">
                        <Heading :level="3" class="c-download-group__title">{{ group.console }}</Heading>
                    </div>

                    <div class="c-download-grid">
                        <div v-for="se in group.emulators" :key="se.id" class="c-download-card">
                            <div class="c-download-card__top">
                                <div>
                                    <Heading :level="4" class="c-download-card__title">{{ se.name }}</Heading>
                                    <div class="c-tag-list">
                                        <span
                                            v-for="c in se.consoles"
                                            :key="c"
                                            class="c-tag"
                                            :style="{
                                                background: consoleStore.getConsoleColor(c.toLowerCase())
                                                    ? `${consoleStore.getConsoleColor(c.toLowerCase())}33`
                                                    : 'rgba(255, 255, 255, 0.03)',
                                                color:
                                                    consoleStore.getConsoleColor(c.toLowerCase()) ||
                                                    'var(--color-text-muted)',
                                                borderColor: consoleStore.getConsoleColor(c.toLowerCase())
                                                    ? `${consoleStore.getConsoleColor(c.toLowerCase())}66`
                                                    : 'rgba(255, 255, 255, 0.08)',
                                            }">
                                            {{ c.toUpperCase() }}
                                        </span>
                                    </div>
                                </div>
                            </div>

                            <div class="c-download-card__bottom">
                                <div class="c-download-card__metadata">
                                    <Text
                                        v-if="formatBytes(se.file_size)"
                                        variant="muted"
                                        size="sm"
                                        class="c-download-card__size">
                                        {{ formatBytes(se.file_size) }}
                                    </Text>
                                    <Text
                                        v-if="se.source_server"
                                        variant="muted"
                                        size="xs"
                                        class="c-download-card__domain"
                                        :title="se.source_server">
                                        {{ se.source_server.replace(/^https?:\/\//, "").replace(/\/$/, "") }}
                                    </Text>
                                </div>
                                <Button @click="emit('download', se)" :disabled="loading" color="primary" size="sm">
                                    <template v-if="loading">Queuing...</template>
                                    <template v-else>Download</template>
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-download-modal {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    width: 100%;
    min-width: 600px;
    max-width: 800px;

    &__search-wrap {
        position: relative;
        width: 100%;
    }

    &__search-icon {
        position: absolute;
        left: 14px;
        top: 50%;
        transform: translateY(-50%);
        color: var(--color-text-muted);
        opacity: 0.6;
    }

    &__search {
        width: 100%;
        padding: 12px 14px 12px 42px;
        border-radius: var(--radius-md);
        border: 1px solid var(--color-border);
        background: rgba(255, 255, 255, 0.03);
        color: var(--color-text);
        font-size: 0.95rem;
        outline: none;
        transition:
            border-color 0.2s,
            background 0.2s;

        &:focus {
            border-color: var(--color-primary);
            background: rgba(255, 255, 255, 0.06);
        }

        &::placeholder {
            color: var(--color-text-muted);
            opacity: 0.5;
        }
    }

    &__loading,
    &__empty {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-xxl) 0;
    }

    &__content {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xl);
        max-height: 60vh;
        overflow-y: auto;
        padding-right: 8px;

        &::-webkit-scrollbar {
            width: 6px;
        }
        &::-webkit-scrollbar-track {
            background: transparent;
        }
        &::-webkit-scrollbar-thumb {
            background: var(--color-scrollbar-thumb);
            border-radius: 10px;
        }
    }
}

.c-download-group {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);

    &__header {
        display: flex;
        align-items: center;
        margin-bottom: 4px;
    }

    &__title {
        margin: 0;
        font-size: 1.15rem;
        font-weight: 800;
        letter-spacing: 0.5px;
        color: var(--color-text-muted);
    }
}

.c-download-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--spacing-md);
}

.c-download-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: var(--spacing-md) var(--spacing-lg);
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);

    &__top {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: 12px;
        margin-bottom: 16px;
    }

    &__title {
        margin: 0;
        font-size: 1.05rem;
        font-weight: 700;
        line-height: 1.3;
    }

    &__bottom {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-top: auto;
    }

    &__metadata {
        display: flex;
        flex-direction: column;
        gap: 2px;
        overflow: hidden;
        margin-right: 8px;
    }

    &__size {
        font-variant-numeric: tabular-nums;
        font-weight: 600;
        opacity: 0.7;
    }

    &__domain {
        opacity: 0.5;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        font-size: 0.65rem;
        letter-spacing: 0.5px;
    }
}

.c-tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
}

.c-tag {
    font-size: 0.7rem;
    font-weight: 800;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: var(--color-text-muted);
    background: rgba(255, 255, 255, 0.03);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    user-select: none;
}
</style>
