<script setup lang="ts">
import { Download } from "lucide-vue-next";
import { computed } from "vue";
import Button from "../ui/Button.vue";
import Modal from "../ui/Modal.vue";
import Text from "../ui/Text.vue";

export type InstallItem = {
    name: string;
    description?: string;
    size: number;
    type: "game" | "emulator";
};

const props = defineProps<{
    show: boolean;
    title?: string;
    items: InstallItem[];
    loading?: boolean;
}>();

const emit = defineEmits(["close", "confirm"]);

const totalSize = computed(() => {
    return props.items.reduce((acc, item) => acc + item.size, 0);
});

const formatBytes = (bytes: number, decimals = 2) => {
    if (!+bytes) return "0 Bytes";
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
};
</script>

<template>
    <Modal :show="show" :title="title || 'Confirm Installation'" @close="emit('close')">
        <div class="c-install-modal">
            <div class="c-install-modal__content">
                <div class="c-install-modal__info">
                    <Text variant="muted"> Are you sure you want to download the following files? </Text>

                    <slot name="header"></slot>

                    <div class="c-install-modal__breakdown">
                        <div v-for="item in items" :key="item.name" class="c-install-modal__item">
                            <div class="c-install-modal__item-label">
                                <Text size="sm" font-weight="bold">{{ item.name }}</Text>
                                <Text v-if="item.description" size="xs" variant="muted">{{ item.description }}</Text>
                            </div>
                            <Text size="sm" font-weight="bold">{{ formatBytes(item.size) }}</Text>
                        </div>

                        <div class="c-install-modal__total">
                            <Text size="sm" font-weight="bold">Total Download Size</Text>
                            <Text size="sm" font-weight="black" color="primary">{{ formatBytes(totalSize) }}</Text>
                        </div>
                    </div>
                </div>
            </div>

            <div class="c-install-modal__actions">
                <Button color="grey" @click="emit('close')">Cancel</Button>
                <Button color="primary" @click="emit('confirm')" :loading="loading">
                    <Download :size="16" /> Install
                </Button>
            </div>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-install-modal {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    padding: var(--spacing-sm) 0;

    &__content {
        display: flex;
        gap: var(--spacing-lg);
        align-items: flex-start;
    }

    &__icon-wrap {
        background: var(--color-primary-light);
        color: var(--color-primary);
        padding: var(--spacing-md);
        border-radius: var(--radius-lg);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    &__info {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
        flex: 1;
    }

    &__breakdown {
        margin-top: var(--spacing-sm);
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs);
        padding: var(--spacing-md);
        background: var(--color-surface-variant);
        border-radius: var(--radius-md);
        border: 1px solid var(--color-border);
    }

    &__item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding-bottom: var(--spacing-xs);
        border-bottom: 1px solid var(--color-border);

        &:last-of-type {
            border-bottom: none;
        }
    }

    &__item-label {
        display: flex;
        flex-direction: column;
    }

    &__total {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-top: var(--spacing-xs);
        padding-top: var(--spacing-sm);
    }

    &__actions {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-md);
        width: 100%;

        :deep(.c-button) {
            width: 100%;
        }
    }
}
</style>
