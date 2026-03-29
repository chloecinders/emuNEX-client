<script setup lang="ts">
import { ChevronRight } from "lucide-vue-next";
import Heading from "../ui/Heading.vue";
import Text from "../ui/Text.vue";

defineProps<{
    consoleName: string;
    count: number;
    totalSize: string;
    color: string;
}>();

const emit = defineEmits<{
    (e: "click"): void;
}>();
</script>

<template>
    <div class="c-console-card" @click="emit('click')">
        <div
            class="c-console-card__bg-indicator"
            :style="{ backgroundColor: color || 'var(--color-primary)' }"
        />

        <div class="c-console-card__content">
            <div class="c-console-card__info">
                <div class="c-console-card__badge-wrapper">
                    <Heading :level="2" color="primary" class="c-console-card__badge">
                        {{ consoleName.toUpperCase() }}
                        <span class="c-console-card__count">{{ count }} Installed</span>
                    </Heading>
                </div>
                <Text variant="muted" size="sm">Total space used: {{ totalSize }}</Text>
            </div>
            <ChevronRight :size="24" class="c-console-card__arrow" />
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-console-card {
    position: relative;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    cursor: pointer;
    transition: all 0.25s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    box-shadow: var(--shadow-sm);
    display: flex;
    flex-direction: column;

    &:hover {
        border-color: var(--color-primary-light);
        box-shadow: var(--shadow-md);
        transform: translateY(-2px);

        .c-console-card__arrow {
            transform: translateX(4px);
            color: var(--color-primary);
        }
    }

    &__bg-indicator {
        height: 6px;
        width: 100%;
        opacity: 0.8;
    }

    &__content {
        padding: var(--spacing-lg);
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-grow: 1;
    }

    &__badge-wrapper {
        margin-bottom: var(--spacing-sm);
    }

    &__count {
        font-size: 0.75rem;
        font-weight: 800;
        color: var(--color-text-muted);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    &__info {
        display: flex;
        flex-direction: column;
    }

    &__arrow {
        color: var(--color-text-muted);
        transition: all 0.2s ease;
    }
}
</style>
