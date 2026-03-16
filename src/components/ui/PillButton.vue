<script setup lang="ts">
const emit = defineEmits(["click"]);
</script>

<template>
    <button class="c-pill-btn" @click.stop="emit('click', $event)">
        <slot />
    </button>
</template>

<style lang="scss" scoped>
.c-pill-btn {
    background: var(--color-surface-variant);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-full);
    padding: 8px 16px;
    font-weight: 800;
    color: var(--color-primary);
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.175, 0.885, 0.32, 1);
    display: flex;
    align-items: center;
    gap: 8px;

    &:hover {
        border-color: var(--color-primary);
        background: var(--color-surface);
        transform: translateY(-2px);
    }

    :deep(svg) {
        width: 20px;
        height: 20px;
        stroke: var(--color-primary);
        stroke-width: 2.5px;
    }
}
</style>
