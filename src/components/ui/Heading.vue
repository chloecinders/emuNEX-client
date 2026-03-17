<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
    defineProps<{
        level?: 1 | 2 | 3 | 4 | 5 | 6 | "1" | "2" | "3" | "4" | "5" | "6" | "badge";
        class?: string;
        isBadge?: boolean;
        color?: "primary" | "normal" | "secondary";
    }>(),
    { color: "normal" },
);

const tag = computed(() => `h${props.level || 2}`);
</script>

<template>
    <div v-if="isBadge" class="c-heading__badge-container">
        <component :is="tag" :class="['c-heading', `c-heading--${color}`, `c-heading--h${level || 2}`, props.class]">
            <slot></slot>
        </component>
    </div>

    <component v-else :is="tag" :class="['c-heading', `c-heading--h${level || 2}`, props.class]">
        <slot></slot>
    </component>
</template>

<style lang="scss" scoped>
.c-heading {
    display: flex;
    flex-direction: row;
    gap: var(--spacing-sm);
    align-items: center;
    margin: 0;
    font-weight: 800;
    line-height: 1.2;

    &--primary {
        color: var(--color-primary);
    }

    &--normal {
        color: var(--color-text);
    }

    &--secondary {
        color: var(--color-text-muted);
    }

    &--h1 {
        font-size: 2.5rem;
        letter-spacing: -1.5px;
    }

    &--h2 {
        font-size: 1.5rem;
        letter-spacing: -0.5px;
    }

    &--h3 {
        font-size: 1.25rem;
    }

    &--h4 {
        font-size: 1.1rem;
    }

    &--h5 {
        font-size: 1rem;
    }

    &--h6 {
        font-size: 0.875rem;
        text-transform: uppercase;
        letter-spacing: 1px;
    }

    &__badge-container {
        display: inline-flex;
        align-items: center;
        gap: var(--spacing-md);
        background: var(--color-surface-variant);
        padding: var(--spacing-sm) var(--spacing-md);
        border-radius: var(--radius-full);
        border: 1px solid var(--color-border);
        width: fit-content;
    }

    &--hbadge {
        font-size: 1.1rem;
        font-weight: 800;
        margin: 0;
    }
}
</style>
