<script setup lang="ts">
interface Props {
    modelValue: boolean;
    label?: string;
    disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
    modelValue: false,
    disabled: false,
});

const emit = defineEmits(["update:modelValue"]);

const toggle = () => {
    if (!props.disabled) {
        emit("update:modelValue", !props.modelValue);
    }
};
</script>

<template>
    <div class="c-switch-field" :class="{ 'c-switch-field--disabled': disabled }" @click="toggle">
        <label v-if="label" class="c-switch-field__label">{{ label }}</label>

        <div class="c-switch" :class="{ 'c-switch--active': modelValue }">
            <div class="c-switch__track">
                <div class="c-switch__handle">
                    <div class="c-switch__handle-inner"></div>
                </div>
            </div>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-switch-field {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--spacing-md);
    cursor: pointer;
    user-select: none;
    padding: var(--spacing-sm) 0;
    transition: opacity 0.2s ease;

    &--disabled {
        cursor: not-allowed;
        opacity: 0.5;
    }

    &__label {
        font-size: 0.9rem;
        font-weight: 700;
        color: var(--color-text);
        cursor: inherit;
    }
}

.c-switch {
    position: relative;
    width: 48px;
    height: 26px;
    transition: all 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);

    &__track {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: var(--color-surface-variant);
        border: 2px solid var(--color-border);
        border-radius: var(--radius-full);
        transition: all 0.3s ease;
        box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.05);
    }

    &__handle {
        position: absolute;
        top: 2px;
        left: 2px;
        width: 18px;
        height: 18px;
        background: white;
        border-radius: 50%;
        box-shadow: var(--shadow-subtle);
        transition: transform 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 2;
    }

    &__handle-inner {
        width: 6px;
        height: 6px;
        background: var(--color-border);
        border-radius: 50%;
        transition: all 0.3s ease;
    }

    &--active {
        .c-switch__track {
            background: var(--color-primary);
            border-color: var(--color-primary-dark);
            box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
        }

        .c-switch__handle {
            transform: translateX(22px);
            box-shadow: 0 2px 8px rgba(var(--color-primary-rgb), 0.3);
        }

        .c-switch__handle-inner {
            background: var(--color-primary);
            width: 8px;
            height: 8px;
        }
    }

    &:hover:not(.c-switch-field--disabled *) {
        .c-switch__track {
            border-color: var(--color-primary-light);
        }
        
        &.c-switch--active .c-switch__track {
            filter: brightness(110%);
        }
    }
}
</style>
