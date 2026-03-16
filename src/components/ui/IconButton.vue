<script setup lang="ts">
defineProps<{
    color?: "default" | "red";
    title?: string;
}>();

const emit = defineEmits(["click"]);
</script>

<template>
    <button
        class="c-icon-btn"
        :class="{ 'c-icon-btn--red': color === 'red' }"
        :title="title"
        @click.stop="emit('click', $event)"
    >
        <slot />
    </button>
</template>

<style lang="scss" scoped>
.c-icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 8px;
    border-radius: var(--radius-sm);
    transition: all 0.2s ease;

    &:hover {
        color: var(--color-primary);
        background: color-mix(in srgb, var(--color-primary) 10%, transparent);
    }

    &--red {
        &:hover {
            color: var(--color-danger);
            background: rgba(255, 68, 68, 0.1);
        }
    }

    :deep(svg) {
        width: 20px;
        height: 20px;
        pointer-events: none;
        stroke-width: 2.5px;
    }
}
</style>
