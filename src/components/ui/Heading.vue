<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
    level?: 1 | 2 | 3 | 4 | 5 | 6;
    class?: string;
}>();

const tag = computed(() => `h${props.level || 2}`);
</script>

<template>
    <component :is="tag" :class="['c-heading', `c-heading--h${level || 2}`, props.class]">
        <slot></slot>
    </component>
</template>

<style lang="scss" scoped>
.c-heading {
    margin: 0;
    font-weight: 800;
    color: var(--color-text);
    line-height: 1.2;

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
}
</style>
