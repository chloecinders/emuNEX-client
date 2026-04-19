<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";

interface Option {
    name: string;
    value: string;
}

const props = defineProps<{
    modelValue: string;
    options: Option[];
    placeholder?: string;
    label?: string;
}>();

const emit = defineEmits(["update:modelValue"]);

const isOpen = ref(false);
const selectRef = ref<HTMLElement | null>(null);

function toggle() {
    isOpen.value = !isOpen.value;
}

function selectOption(value: string) {
    emit("update:modelValue", value);
    isOpen.value = false;
}

function handleClickOutside(event: MouseEvent) {
    if (selectRef.value && !selectRef.value.contains(event.target as Node)) {
        isOpen.value = false;
    }
}

onMounted(() => {
    document.addEventListener("mousedown", handleClickOutside);
});

onUnmounted(() => {
    document.removeEventListener("mousedown", handleClickOutside);
});

const selectedLabel = computed(() => {
    const selected = props.options.find((o) => o.value === props.modelValue);
    return selected ? selected.name : props.placeholder || "Select...";
});
</script>

<template>
    <div class="c-select" ref="selectRef">
        <label v-if="label" class="c-select__label">{{ label }}</label>

        <div 
            class="c-select__wrapper" 
            :class="{ 'c-select__wrapper--open': isOpen }" 
            @click="toggle"
            tabindex="0"
            role="combobox"
            :aria-expanded="isOpen"
            @keydown.enter.space.prevent="toggle"
        >
            <div class="c-select__display">
                <span class="c-select__display-text" :class="{ 'c-select__display-text--placeholder': !modelValue }">
                    {{ selectedLabel }}
                </span>
                <span class="c-select__arrow">
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                        <path
                            d="M2 4L6 8L10 4"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        />
                    </svg>
                </span>
            </div>
 
            <transition name="pop">
                <div v-if="isOpen" class="c-select__menu">
                    <div
                        v-for="option in options"
                        :key="option.value"
                        class="c-select__option"
                        :class="{ 'c-select__option--selected': option.value === modelValue }"
                        @click.stop="selectOption(option.value)"
                        tabindex="0"
                        role="option"
                        @keydown.enter.space.prevent="selectOption(option.value)"
                    >
                        {{ option.name }}
                    </div>
                </div>
            </transition>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-select {
    margin-bottom: var(--spacing-md);
    width: 100%;
    position: relative;
    user-select: none;

    &__label {
        display: block;
        font-size: 0.8rem;
        font-weight: 800;
        color: var(--color-text-muted);
        margin-bottom: var(--spacing-xs);
        margin-left: var(--spacing-xs);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    &__wrapper {
        position: relative;
        cursor: pointer;
        background: var(--color-surface);
        border: 2px solid var(--color-border);
        border-radius: var(--radius-md);
        transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);

        &:hover {
            border-color: var(--color-primary);
        }

        &--open {
            border-color: var(--color-primary);
            box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.1);

            .c-select__arrow {
                transform: rotate(180deg);
            }
        }
    }

    &__display {
        padding: 12px 16px;
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: var(--spacing-md);
    }

    &__display-text {
        font-size: 1rem;
        font-weight: 600;
        color: var(--color-text);

        &--placeholder {
            opacity: 0.5;
        }
    }

    &__arrow {
        color: var(--color-primary);
        transition: transform 0.2s ease;
    }

    &__menu {
        position: absolute;
        top: calc(100% + 8px);
        left: 0;
        right: 0;
        background: var(--color-surface);
        border: 2px solid var(--color-primary);
        border-radius: var(--radius-md);
        box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
        z-index: 1000;
        overflow: hidden;
        max-height: 240px;
        overflow-y: auto;
    }

    &__option {
        padding: 12px 16px;
        font-size: 0.95rem;
        font-weight: 700;
        color: var(--color-text);
        transition: all 0.1s ease;

        &:hover,
        &:focus {
            background: var(--color-primary);
            color: white;
            outline: none;
            box-shadow: none !important;
        }

        &--selected {
            background: var(--color-surface-variant);
            color: var(--color-primary);

            &:hover,
            &:focus {
                background: var(--color-primary);
                color: white;
            }
        }
    }
}

.pop-enter-active,
.pop-leave-active {
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.pop-enter-from,
.pop-leave-to {
    opacity: 0;
    transform: translateY(-8px) scale(0.98);
}
</style>
