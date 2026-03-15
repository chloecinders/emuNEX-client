<script setup lang="ts">
const props = defineProps<{
    bgColor?: string;
    class?: string;
}>();
</script>

<template>
    <span :class="['c-badge', props.class]" :style="{ background: props.bgColor || 'var(--color-primary)' }">
        <slot></slot>
    </span>
</template>

<style lang="scss" scoped>
.c-badge {
    color: var(--color-text-on-primary);
    font-size: 0.625rem;
    padding: var(--spacing-xxs) var(--spacing-sm);
    border-radius: var(--radius-full);
    font-weight: 900;
    border: 2px solid var(--color-surface);
    box-shadow: var(--shadow-subtle);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
}
</style>
