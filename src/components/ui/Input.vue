<script setup lang="ts">
defineProps<{
    modelValue: string;
    placeholder?: string;
    type?: string;
    label?: string;
}>();

defineEmits(["update:modelValue"]);
</script>

<template>
    <div class="input-container">
        <label v-if="label" class="input-label">{{ label }}</label>

        <div class="input-wrapper">
            <input
                :type="type || 'text'"
                :value="modelValue"
                :placeholder="placeholder"
                class="nintendo-input"
                @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
            />

            <div class="input-shadow"></div>
        </div>
    </div>
</template>

<style scoped>
.input-container {
    margin-bottom: var(--spacing-md);
    width: 100%;
}

.input-label {
    display: block;
    font-size: 0.8rem;
    font-weight: 800;
    color: var(--color-text-muted);
    margin-bottom: var(--spacing-xs);
    margin-left: var(--spacing-xs);
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.input-wrapper {
    position: relative;
    border-radius: var(--radius-md);
    background: var(--color-surface);
}

.nintendo-input {
    width: 100%;
    padding: 12px 16px;
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    background: transparent;
    font-size: 1rem;
    color: var(--color-text);
    outline: none;
    transition: all 0.2s ease;
    box-sizing: border-box;
    font-family: inherit;
    font-weight: 600;
}

.nintendo-input:focus {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.1);
    background: var(--color-surface);
}

.nintendo-input::placeholder {
    color: var(--color-text-muted);
    opacity: 0.5;
}
</style>
