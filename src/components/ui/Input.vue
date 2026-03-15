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
    <div class="c-input-field">
        <label v-if="label" class="c-input-field__label">{{ label }}</label>

        <div class="c-input-field__wrapper">
            <input
                :type="type || 'text'"
                :value="modelValue"
                :placeholder="placeholder"
                class="c-input-field__input"
                @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
            />

            <div class="c-input-field__shadow"></div>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-input-field {
    margin-bottom: var(--spacing-md);
    width: 100%;

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
        border-radius: var(--radius-md);
        background: var(--color-surface);
    }

    &__input {
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

        &:focus {
            border-color: var(--color-primary);
            box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.1);
            background: var(--color-surface);
        }

        &::placeholder {
            color: var(--color-text-muted);
            opacity: 0.5;
        }
    }
}
</style>
