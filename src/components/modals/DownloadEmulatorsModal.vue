<script setup lang="ts">
import { formatBytes } from "../../lib/format";
import type { ServerEmulator } from "../../stores/EmulatorStore";
import Button from "../ui/Button.vue";
import Heading from "../ui/Heading.vue";
import Modal from "../ui/Modal.vue";
import Spinner from "../ui/Spinner.vue";
import Text from "../ui/Text.vue";

defineProps<{
    show: boolean;
    serverEmulators: ServerEmulator[];
    isFetching: boolean;
    loading: boolean;
}>();

const emit = defineEmits<{
    (e: "close"): void;
    (e: "download", serverEmulator: ServerEmulator): void;
}>();
</script>

<template>
    <Modal :show="show" title="Download Emulators" @close="emit('close')">
        <div class="c-download-modal">
            <div v-if="isFetching" class="c-download-modal__loading">
                <Spinner />
                <Text>Fetching available emulators...</Text>
            </div>
            <div v-else-if="serverEmulators.length === 0" class="c-download-modal__empty">
                <Text variant="muted">No emulators available for download on the server.</Text>
            </div>
            <div v-else class="c-download-list">
                <div v-for="se in serverEmulators" :key="se.id" class="c-download-item">
                    <div class="c-download-item__info">
                        <Heading :level="4" class="c-download-item__title">{{ se.name }}</Heading>
                        <div class="c-tag-list" style="margin-top: 0">
                            <span v-for="c in se.consoles" :key="c" class="c-tag">{{ c.toUpperCase() }}</span>
                        </div>

                        <Text
                            v-if="formatBytes(se.file_size)"
                            variant="muted"
                            size="sm"
                            class="c-download-item__size"
                            >{{ formatBytes(se.file_size) }}</Text
                        >
                    </div>
                    <Button @click="emit('download', se)" :disabled="loading" color="primary">
                        <template v-if="loading">Downloading...</template>
                        <template v-else>Download</template>
                    </Button>
                </div>
            </div>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-download-modal {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);

    &__loading,
    &__empty {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-xl) 0;
    }
}

.c-download-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
}

.c-download-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: all 0.2s ease;

    &:hover {
        border-color: var(--color-primary);
        background: var(--color-surface-hover);
    }

    &__info {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs);
    }

    &__title {
        margin: 0;
        font-weight: 700;
    }

    &__size {
        font-variant-numeric: tabular-nums;
    }
}

.c-tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 4px;
}

.c-tag {
    font-size: 0.72rem;
    font-weight: 800;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
    background: var(--color-surface);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    user-select: none;
}
</style>
