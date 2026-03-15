<script setup lang="ts">
const props = defineProps<{
    variant?: "body" | "muted" | "label" | "error" | "success";
    size?: "xs" | "sm" | "md" | "lg";
    class?: string;
}>();
</script>

<template>
    <span :class="['c-text', `c-text--${variant || 'body'}`, `c-text--${size || 'md'}`, props.class]">
        <slot></slot>
    </span>
</template>

<style lang="scss" scoped>
.c-text {
    line-height: 1.5;

    &--body {
        color: var(--color-text);
    }

    &--muted {
        color: var(--color-text-muted);
    }

    &--label {
        color: var(--color-text-muted);
        text-transform: uppercase;
        letter-spacing: 0.5px;
        font-weight: 800;
    }

    &--error {
        color: var(--color-danger);
    }

    &--success {
        color: #2f855a;
    }

    &--xs {
        font-size: 0.75rem;
    }

    &--sm {
        font-size: 0.85rem;
    }

    &--md {
        font-size: 1rem;
    }

    &--lg {
        font-size: 1.125rem;
    }
}
</style>
