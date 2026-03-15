<script setup lang="ts">
const props = defineProps<{
    size?: "sm" | "md" | "lg";
    class?: string;
}>();
</script>

<template>
    <div :class="['c-spinner', `c-spinner--${size || 'md'}`, props.class]"></div>
</template>

<style lang="scss" scoped>
.c-spinner {
    border: var(--spacing-xs) solid var(--color-surface-variant);
    border-top: var(--spacing-xs) solid var(--color-primary);
    border-radius: var(--radius-full);
    animation: spin 0.8s cubic-bezier(0.4, 0, 0.2, 1) infinite;
    display: inline-block;

    &--sm {
        width: var(--spacing-md);
        height: var(--spacing-md);
        border-width: 2px;
    }

    &--md {
        width: var(--spacing-xl);
        height: var(--spacing-xl);
        border-width: 3px;
    }

    &--lg {
        width: var(--spacing-xxl);
        height: var(--spacing-xxl);
        border-width: 4px;
    }
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}
</style>
