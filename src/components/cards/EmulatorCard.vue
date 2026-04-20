<script setup lang="ts">
import { ArrowDownToLine, CheckCircle, MoreVertical, Pencil, RefreshCw, Trash2 } from "lucide-vue-next";
import { onMounted, onUnmounted, ref } from "vue";
import { formatBytes } from "../../lib/format";
import type { Emulator } from "../../stores/EmulatorStore";
import Heading from "../ui/Heading.vue";
import IconButton from "../ui/IconButton.vue";
import Text from "../ui/Text.vue";

defineProps<{
    emulator: Emulator;
    dirSize?: number;
    hasUpdate?: boolean;
}>();

const emit = defineEmits(["edit", "delete", "setDefault", "refreshConfig", "update"]);

const showMenu = ref(false);

const toggleMenu = (e: Event) => {
    e.stopPropagation();
    showMenu.value = !showMenu.value;
};

const closeMenu = (e: Event) => {
    const target = e.target as HTMLElement;
    if (!target.closest(".c-emulator-card__actions")) {
        showMenu.value = false;
    }
};

onMounted(() => {
    window.addEventListener("click", closeMenu);
});

onUnmounted(() => {
    window.removeEventListener("click", closeMenu);
});

const handleAction = (action: "edit" | "delete" | "setDefault" | "refreshConfig") => {
    emit(action);
    showMenu.value = false;
};
</script>

<template>
    <div
        class="c-emulator-card"
        :class="{
            'c-emulator-card--default': emulator.is_default,
            'c-emulator-card--menu-open': showMenu,
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
                <div class="c-emulator-card__meta">
                    <Text v-if="dirSize" variant="muted" size="sm">{{ formatBytes(dirSize) }}</Text>
                    <Text v-if="emulator.source_server" variant="muted" size="sm">
                        {{ dirSize ? " • " : "" }}
                        {{ emulator.source_server.replace(/^https?:\/\//, "").replace(/\/$/, "") }}
                    </Text>
                    <Text v-if="emulator.version" variant="muted" size="sm">
                        {{ dirSize || emulator.source_server ? " • " : "" }}
                        v{{ emulator.version }}
                    </Text>
                </div>
            </div>

            <div class="c-emulator-card__actions">
                <IconButton v-if="hasUpdate" @click.stop="emit('update')" class="c-emulator-card__update-icon">
                    <ArrowDownToLine />
                </IconButton>
                <IconButton @click="toggleMenu" :class="{ 'is-active': showMenu }">
                    <MoreVertical />
                </IconButton>

                <transition name="fade">
                    <div v-if="showMenu" class="c-emulator-card__menu">
                        <button
                            v-if="!emulator.is_default"
                            class="c-emulator-card__menu-item"
                            @click="handleAction('setDefault')"
                        >
                            <CheckCircle :size="16" /> Make Default
                        </button>
                        <button
                            v-if="!emulator.id.startsWith('custom-')"
                            class="c-emulator-card__menu-item"
                            @click="handleAction('refreshConfig')"
                        >
                            <RefreshCw :size="16" /> Refresh Config
                        </button>
                        <button class="c-emulator-card__menu-item" @click="handleAction('edit')">
                            <Pencil :size="16" /> Edit
                        </button>
                        <button
                            class="c-emulator-card__menu-item c-emulator-card__menu-item--danger"
                            @click="handleAction('delete')"
                        >
                            <Trash2 :size="16" /> Delete
                        </button>
                    </div>
                </transition>
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

    &--menu-open {
        z-index: 100;
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
        gap: 4px;
        flex: 1;
    }

    &__meta {
        display: flex;
        align-items: center;
        gap: 2px;
    }

    &__update-icon {
        color: var(--color-primary);
        background: color-mix(in srgb, var(--color-primary) 10%, transparent);

        &:hover {
            background: var(--color-primary);
            color: var(--color-surface);
        }
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
        position: relative;
    }

    &__menu {
        position: absolute;
        top: 100%;
        right: 0;
        margin-top: 4px;
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        box-shadow: var(--shadow-lg);
        min-width: 160px;
        display: flex;
        flex-direction: column;
        padding: 4px;
        z-index: 100;
    }

    &__menu-item {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        font-family: inherit;
        font-size: 0.85rem;
        font-weight: 600;
        color: var(--color-text);
        background: transparent;
        border: none;
        border-radius: var(--radius-sm);
        cursor: pointer;
        text-align: left;
        transition:
            background-color 0.15s,
            color 0.15s;

        &:hover,
        &:focus {
            background: var(--color-surface-variant);
            outline: none;
        }

        &--danger {
            color: var(--color-danger);
        }

        &--danger:hover,
        &--danger:focus {
            background: color-mix(in srgb, var(--color-danger) 15%, transparent);
            color: var(--color-danger);
        }
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
