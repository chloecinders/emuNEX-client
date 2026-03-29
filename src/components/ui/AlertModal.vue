<script setup lang="ts">
import Button, { ButtonColor } from "./Button.vue";
import Modal from "./Modal.vue";

withDefaults(
    defineProps<{
        show: boolean;
        title: string;
        subtitle?: string;
        message?: string;
        cancelLabel?: string;
        confirmLabel?: string;
        confirmColor?: ButtonColor;
        cancelColor?: ButtonColor;
        width?: string;
    }>(),
    {
        cancelLabel: "Cancel",
        confirmLabel: "Confirm",
        confirmColor: "red",
        cancelColor: "grey",
        width: "400px",
    },
);

const emit = defineEmits<{
    (e: "close"): void;
    (e: "confirm"): void;
}>();
</script>

<template>
    <Modal :show="show" :title="title" :subtitle="subtitle" :width="width" @close="emit('close')">
        <div class="c-alert-modal">
            <slot>
                <p v-if="message" class="c-alert-modal__message" v-html="message" />
            </slot>

            <div class="c-alert-modal__actions">
                <Button :color="cancelColor" @click="emit('close')">{{ cancelLabel }}</Button>
                <Button :color="confirmColor" @click="emit('confirm')">{{ confirmLabel }}</Button>
            </div>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-alert-modal {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) 0;

    &__message {
        margin: 0;
        font-size: 0.95rem;
        color: var(--color-text);
        line-height: 1.55;
    }

    &__actions {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-md);
        width: 100%;
        margin-top: var(--spacing-sm);

        :deep(.c-button) {
            width: 100%;
        }
    }
}
</style>
