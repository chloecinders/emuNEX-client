<script setup lang="ts">
export type ButtonColor = "primary" | "green" | "red" | "grey";

interface Props {
    color?: ButtonColor;
    size?: "sm" | "md";
    disabled?: boolean;
    full?: boolean;
}

withDefaults(defineProps<Props>(), {
    color: "primary",
    size: "md",
    disabled: false,
    full: false,
});

const emit = defineEmits(["click"]);
</script>

<template>
    <button
        class="c-button"
        :class="[`c-button--${color}`, `c-button--${size}`, { 'c-button--disabled': disabled, 'c-button--full': full }]"
        :disabled="disabled"
        @click="emit('click', $event)"
    >
        <span class="c-button__edge"></span>
        <span class="c-button__front">
            <slot />
        </span>
    </button>
</template>

<style lang="scss" scoped>
.c-button {
    position: relative;
    border: none;
    background: transparent;
    padding: 0;
    cursor: pointer;
    outline-offset: 4px;
    transition: filter 250ms;
    user-select: none;
    touch-action: manipulation;
    font-family: inherit;
    font-weight: 800;
    font-size: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    display: inline-block;

    &--full {
        width: 100%;
        height: 100%;
    }

    &__front {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        position: relative;
        top: -6px;
        width: 100%;
        height: 100%;
        padding: 12px 28px;
        border-radius: var(--radius-md);
        color: white;
        transition:
            transform 150ms cubic-bezier(0.3, 0.7, 0.4, 1),
            top 150ms cubic-bezier(0.3, 0.7, 0.4, 1);
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
        border: 1px solid rgba(255, 255, 255, 0.1);
        user-select: none;
        z-index: 2;
    }

    &__edge {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        border-radius: var(--radius-md);
        z-index: 1;
    }

    &--primary {
        .c-button__front {
            background: var(--color-primary);
        }
        .c-button__edge {
            background: var(--color-primary-dark);
        }
    }

    &--green {
        .c-button__front {
            background: #4caf50;
        }
        .c-button__edge {
            background: #3d8b40;
        }
    }

    &--red {
        .c-button__front {
            background: #e60012;
        }
        .c-button__edge {
            background: #b3000e;
        }
    }

    &--grey {
        .c-button__front {
            background: var(--color-border);
            color: var(--color-text);
            text-shadow: none;
            border: 1px solid var(--color-border-hover);
        }
        .c-button__edge {
            background: var(--color-text-muted);
        }

        [data-theme="dark"] & {
            .c-button__front {
                background: #4a4a5a;
                color: #ffffff;
                border-color: #5b5b6a;
            }
            .c-button__edge {
                background: #2b2b36;
            }
        }
    }

    &--sm {
        font-size: 0.8rem;
        .c-button__front {
            padding: 8px 16px;
            top: -4px;
        }
    }

    &:hover .c-button__front {
        top: -7px;
        filter: brightness(110%);
    }

    &:active .c-button__front {
        top: 0;
        transition: top 34ms;
    }

    &--disabled {
        cursor: not-allowed;
        filter: grayscale(0.8);
        opacity: 0.7;

        .c-button__front {
            top: 0 !important;
        }
    }
}
</style>
