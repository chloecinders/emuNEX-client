<script lang="ts" setup>
import Button from "../ui/Button.vue";
import Modal from "../ui/Modal.vue";

defineProps<{
    show: boolean;
    version: number | null;
}>();

const emit = defineEmits(["choice"]);
</script>

<template>
    <Modal
        :show="show"
        title="Sync Conflict"
        :subtitle="`Cloud version v${version} is newer than yours.`"
        @close="emit('choice', false)"
    >
        <div class="c-save-conflict">
            <div class="c-save-conflict__icon-group">
                <span class="c-save-conflict__warning-icon">!</span>
            </div>
            <p class="c-save-conflict__description">
                Would you like to download the newer cloud save and overwrite your local progress? 
                <strong>This cannot be undone.</strong>
            </p>
        </div>

        <div class="c-save-conflict__actions">
            <Button color="grey" @click="emit('choice', false)">
                Keep Local
            </Button>
            <Button color="blue" @click="emit('choice', true)">
                Download Cloud
            </Button>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-save-conflict {
    text-align: center;
    margin: var(--spacing-md) 0 var(--spacing-xl) 0;

    &__icon-group {
        margin-bottom: var(--spacing-lg);
        display: flex;
        justify-content: center;
    }

    &__warning-icon {
        width: var(--spacing-xxl);
        height: var(--spacing-xxl);
        background: var(--color-primary);
        color: white;
        font-weight: 950;
        border-radius: var(--radius-md);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.5rem;
    }

    &__description {
        font-size: 0.95rem;
        color: var(--color-text-muted);
        line-height: 1.6;
        font-weight: 500;
        
        strong {
            color: var(--color-text);
            font-weight: 800;
        }
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
