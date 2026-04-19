<script setup lang="ts">
import { CheckCircle, Pencil, Trash2 } from "lucide-vue-next";
import { formatBytes } from "../../lib/format";
import type { Emulator } from "../../stores/EmulatorStore";
import Heading from "../ui/Heading.vue";
import IconButton from "../ui/IconButton.vue";
import Text from "../ui/Text.vue";
import Tooltip from "../ui/Tooltip.vue";

defineProps<{
    emulator: Emulator;
    dirSize?: number;
}>();

const emit = defineEmits<{
    (e: "edit"): void;
    (e: "delete"): void;
    (e: "setDefault"): void;
}>();
</script>

<template>
    <div
        class="c-emulator-card"
        :class="{
            'c-emulator-card--default': emulator.is_default,
        }"
        tabindex="0"
        role="button"
    >
        <div class="c-emulator-card__header">
            <div class="c-emulator-card__title-area">
                <div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap">
                    <Heading :level="3" class="c-emulator-card__name">{{
                        emulator.name || "Unnamed Emulator"
                    }}</Heading>
                    <div v-if="emulator.is_default" class="c-emulator-card__default-tag">
                        <CheckCircle :size="14" /> Default
                    </div>
                </div>
                <Text v-if="dirSize" variant="muted" size="sm">{{ formatBytes(dirSize) }}</Text>
            </div>

            <div class="c-emulator-card__actions">
                <Tooltip v-if="!emulator.is_default" text="Make Default">
                    <IconButton @click="emit('setDefault')">
                        <CheckCircle />
                    </IconButton>
                </Tooltip>

                <Tooltip text="Edit">
                    <IconButton @click="emit('edit')">
                        <Pencil />
                    </IconButton>
                </Tooltip>

                <Tooltip text="Delete">
                    <IconButton color="red" @click="emit('delete')">
                        <Trash2 />
                    </IconButton>
                </Tooltip>
            </div>
        </div>

        <div class="c-emulator-card__body">
            <div class="c-emulator-field">
                <Text variant="label" size="sm">Assigned Consoles</Text>
                <div class="c-tag-list">
                    <span v-for="c in emulator.consoles" :key="c" class="c-tag">{{ c.toUpperCase() }}</span>
                </div>
            </div>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-emulator-card {
    background: var(--color-surface-variant);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    transition: all 0.25s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    box-shadow: var(--shadow-sm);
    position: relative;

    &:hover {
        border-color: var(--color-primary);
        box-shadow: var(--shadow-md);
        transform: translateY(-2px);
    }

    &--default {
        background: var(--color-surface-variant);
    }

    &__header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: var(--spacing-md);
        margin-bottom: var(--spacing-xs);
    }

    &__default-tag {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 0.72rem;
        font-weight: 800;
        color: var(--color-primary);
        background: color-mix(in srgb, var(--color-primary) 12%, transparent);
        padding: 4px 10px;
        border-radius: var(--radius-full);
        border: 1px solid color-mix(in srgb, var(--color-primary) 30%, transparent);
    }

    &__title-area {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: var(--spacing-xs);
        flex: 1;
    }

    &__name {
        margin: 0;
        font-weight: 800;
        color: var(--color-text);
        line-height: 1.2;
    }

    &__actions {
        display: flex;
        gap: var(--spacing-xs);
        flex-wrap: wrap;
        justify-content: flex-end;
    }

    &__body {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-md);
        flex-grow: 1;
    }
}

.c-emulator-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);

    &__value {
        font-family: inherit;
        font-size: 0.85rem;
        word-break: break-all;
        background: var(--color-surface-variant);
        padding: var(--spacing-sm);
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border);
        color: var(--color-text-muted);
    }
}

.c-tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 4px;
}

.c-tag {
    font-size: 0.72rem;
    font-weight: 800;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
    background: var(--color-surface);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    transition: all 0.2s;
    user-select: none;
}
</style>
