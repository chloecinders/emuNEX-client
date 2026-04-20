<script lang="ts" setup>
import { CheckCircle, Info, TriangleAlert, X, XCircle } from "lucide-vue-next";
import { useToast } from "../../lib/useToast";

const { toasts, dismiss } = useToast();

const icons = {
    success: CheckCircle,
    error: XCircle,
    warning: TriangleAlert,
    info: Info,
} as const;
</script>

<template>
    <Teleport to="body">
        <div class="c-toast-stack" aria-live="polite" aria-atomic="false">
            <TransitionGroup name="toast" tag="div" class="c-toast-stack__inner">
                <div
                    v-for="toast in toasts"
                    :key="toast.id"
                    class="c-toast"
                    :class="`c-toast--${toast.variant}`"
                    role="alert"
                >
                    <component :is="icons[toast.variant]" class="c-toast__icon" :size="18" />
                    <span class="c-toast__message">{{ toast.message }}</span>
                    <button class="c-toast__close" @click="dismiss(toast.id)" aria-label="Dismiss">
                        <X :size="14" />
                    </button>
                </div>
            </TransitionGroup>
        </div>
    </Teleport>
</template>

<style lang="scss" scoped>
.c-toast-stack {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 99999;
    pointer-events: none;

    &__inner {
        display: flex;
        flex-direction: column;
        gap: 10px;
        align-items: flex-end;
    }
}

.c-toast {
    pointer-events: all;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    background: var(--color-surface);
    box-shadow:
        0 4px 24px rgba(0, 0, 0, 0.18),
        0 1px 4px rgba(0, 0, 0, 0.1);
    max-width: 380px;
    min-width: 240px;
    font-size: 0.875rem;
    font-weight: 600;
    line-height: 1.4;

    &__icon {
        flex-shrink: 0;
    }

    &__message {
        flex: 1;
        color: var(--color-text);
    }

    &__close {
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 22px;
        height: 22px;
        border-radius: var(--radius-full);
        border: none;
        background: transparent;
        color: var(--color-text-muted);
        cursor: pointer;
        padding: 0;
        transition:
            background 0.15s,
            color 0.15s;

        &:hover {
            background: var(--color-surface-variant);
            color: var(--color-text);
        }
    }

    &--success {
        border-color: oklch(0.7 0.18 140 / 0.35);
        background: color-mix(in srgb, oklch(0.7 0.18 140) 8%, var(--color-surface));

        .c-toast__icon {
            color: oklch(0.7 0.18 140);
        }
    }

    &--error {
        border-color: oklch(0.6 0.2 25 / 0.35);
        background: color-mix(in srgb, oklch(0.6 0.2 25) 8%, var(--color-surface));

        .c-toast__icon {
            color: oklch(0.6 0.2 25);
        }
    }

    &--warning {
        border-color: oklch(0.75 0.18 70 / 0.35);
        background: color-mix(in srgb, oklch(0.75 0.18 70) 8%, var(--color-surface));

        .c-toast__icon {
            color: oklch(0.75 0.18 70);
        }
    }

    &--info {
        border-color: color-mix(in oklch, var(--color-primary) 35%, transparent);
        background: color-mix(in oklch, var(--color-primary) 8%, var(--color-surface));

        .c-toast__icon {
            color: var(--color-primary);
        }
    }
}

.toast-enter-active {
    transition:
        opacity 0.25s ease,
        transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.toast-leave-active {
    transition:
        opacity 0.2s ease,
        transform 0.25s ease;
}

.toast-enter-from {
    opacity: 0;
    transform: translateY(16px) scale(0.95);
}

.toast-leave-to {
    opacity: 0;
    transform: translateX(20px) scale(0.95);
}

.toast-move {
    transition: transform 0.25s ease;
}
</style>
