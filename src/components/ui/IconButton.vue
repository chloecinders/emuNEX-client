<script setup lang="ts">
interface Props {
    title?: string;
    size?: "sm" | "md" | "lg";
}

withDefaults(defineProps<Props>(), {
    size: "md",
});

const emit = defineEmits(["click"]);
</script>

<template>
    <button
        class="c-icon-button"
        :class="[`c-icon-button--${size}`]"
        :title="title"
        @click="emit('click')"
    >
        <slot />
    </button>
</template>

<style lang="scss" scoped>
.c-icon-button {
    flex-shrink: 0;
    background: var(--color-surface-variant);
    border: 2px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: var(--color-primary);
    transition: all 0.2s cubic-bezier(0.175, 0.885, 0.32, 1);
    box-shadow: var(--shadow-subtle);

    &--sm {
        width: 32px;
        height: 32px;
        border-radius: var(--radius-sm);
        
        :deep(svg) {
            width: 16px;
            height: 16px;
        }
    }

    &--md {
        width: 44px;
        height: 44px;
        border-radius: calc(var(--radius-md) - 2px);
        
        :deep(svg) {
            width: 20px;
            height: 20px;
        }
    }

    &--lg {
        width: 60px;
        height: 60px;
        border-radius: var(--radius-md);
        
        :deep(svg) {
            width: 24px;
            height: 24px;
        }
    }

    &:hover {
        background: var(--color-surface);
        border-color: var(--color-primary);
        transform: translateY(-4px) scale(1.05);
        box-shadow: 0 8px 20px rgba(107, 92, 177, 0.2);
        color: var(--color-primary-light);
    }

    &:active {
        transform: scale(0.95);
    }
}
</style>
