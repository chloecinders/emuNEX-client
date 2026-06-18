<script lang="ts" setup>
import Button from "../ui/Button.vue";
import Modal from "../ui/Modal.vue";

defineProps<{
    show: boolean;
}>();

const emit = defineEmits(["confirm", "cancel"]);
</script>

<template>
    <Modal
        :show="show"
        title="Multi-Disc Game"
        subtitle="This game includes multiple disc files and may prompt you to swap discs during gameplay."
        @close="emit('cancel')">
        <div class="c-disc-disclaimer">
            <div class="c-disc-disclaimer__content">
                <p>To swap discs while playing:</p>
                <ol class="c-disc-disclaimer__list">
                    <li>Open your emulator's system menu.</li>
                    <li>Navigate to the change discs section.</li>
                    <li>Select the prompted disc from the playlist.</li>
                </ol>
                <p class="c-disc-disclaimer__note">Note: This disclaimer can be disabled in the client settings.</p>
            </div>
        </div>

        <div class="c-disc-disclaimer__actions">
            <Button color="grey" @click="emit('cancel')">Cancel</Button>
            <Button color="primary" @click="emit('confirm')">Start Game</Button>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-disc-disclaimer {
    margin: var(--spacing-md) 0 var(--spacing-xl) 0;

    &__icon-group {
        margin-bottom: var(--spacing-lg);
        display: flex;
        justify-content: center;
    }

    &__info-icon {
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

    &__content {
        font-size: 0.95rem;
        color: var(--color-text-muted);
        line-height: 1.6;
        font-weight: 500;
        text-align: left;
        padding: 0 var(--spacing-sm);

        p {
            margin: var(--spacing-xs) 0;
        }
    }

    &__list {
        margin: var(--spacing-sm) 0;
        padding-left: var(--spacing-xl);

        li {
            margin-bottom: var(--spacing-xs);
            strong {
                color: var(--color-text);
                font-weight: 700;
            }
        }
    }

    &__note {
        font-size: 0.85rem;
        font-style: italic;
        opacity: 0.8;
        margin-top: var(--spacing-md) !important;
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
