<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";

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
    const selected = props.options.find(o => o.value === props.modelValue);
    return selected ? selected.name : props.placeholder || "Select...";
});
</script>

<template>
    <div class="select-container" ref="selectRef">
        <label v-if="label" class="select-label">{{ label }}</label>

        <div class="select-wrapper" :class="{ 'is-open': isOpen }" @click="toggle">
            <div class="select-display">
                <span class="display-text" :class="{ 'is-placeholder': !modelValue }">
                    {{ selectedLabel }}
                </span>
                <span class="select-arrow">
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                        <path d="M2 4L6 8L10 4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                </span>
            </div>

            <transition name="pop">
                <div v-if="isOpen" class="select-menu">
                    <div 
                        v-for="option in options" 
                        :key="option.value" 
                        class="select-option"
                        :class="{ 'is-selected': option.value === modelValue }"
                        @click.stop="selectOption(option.value)"
                    >
                        {{ option.name }}
                    </div>
                </div>
            </transition>
        </div>
    </div>
</template>

<style scoped>
.select-container {
    margin-bottom: var(--spacing-md);
    width: 100%;
    position: relative;
    user-select: none;
}

.select-label {
    display: block;
    font-size: 0.8rem;
    font-weight: 800;
    color: var(--color-text-muted);
    margin-bottom: var(--spacing-xs);
    margin-left: var(--spacing-xs);
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.select-wrapper {
    position: relative;
    cursor: pointer;
    background: var(--color-surface);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.select-wrapper:hover {
    border-color: var(--color-primary);
}

.select-wrapper.is-open {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.1);
}

.select-display {
    padding: 12px 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-md);
}

.display-text {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text);
}

.display-text.is-placeholder {
    opacity: 0.5;
}

.select-arrow {
    color: var(--color-primary);
    transition: transform 0.2s ease;
}

.is-open .select-arrow {
    transform: rotate(180deg);
}

.select-menu {
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

.select-option {
    padding: 12px 16px;
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--color-text);
    transition: all 0.1s ease;
}

.select-option:hover {
    background: var(--color-primary);
    color: white;
}

.select-option.is-selected {
    background: var(--color-surface-variant);
    color: var(--color-primary);
}

.select-option.is-selected:hover {
    background: var(--color-primary);
    color: white;
}

/* Transitions */
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
