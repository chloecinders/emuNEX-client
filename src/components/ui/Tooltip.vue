<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";

defineProps<{
    text: string;
    disabled?: boolean;
}>();

const isVisible = ref(false);
const tooltipRef = ref<HTMLElement | null>(null);
const triggerRef = ref<HTMLElement | null>(null);
const position = ref({ top: 0, left: 0 });

const updatePosition = () => {
    const element = triggerRef.value?.firstElementChild as HTMLElement;
    if (!element || !tooltipRef.value) return;

    const rect = element.getBoundingClientRect();
    const tooltipHeight = tooltipRef.value.offsetHeight;
    const tooltipWidth = tooltipRef.value.offsetWidth;

    position.value = {
        top: rect.top - tooltipHeight - 8,
        left: rect.left + rect.width / 2 - tooltipWidth / 2,
    };

    if (position.value.top < 8) position.value.top = rect.bottom + 8;
    if (position.value.left < 8) position.value.left = 8;
    else if (position.value.left + tooltipWidth > window.innerWidth - 8) {
        position.value.left = window.innerWidth - tooltipWidth - 8;
    }
};

const showTooltip = () => {
    isVisible.value = true;
    setTimeout(updatePosition, 0);
};

const hideTooltip = () => {
    isVisible.value = false;
};

onMounted(() => {
    window.addEventListener("scroll", updatePosition, true);
    window.addEventListener("resize", updatePosition);
});

onUnmounted(() => {
    window.removeEventListener("scroll", updatePosition, true);
    window.removeEventListener("resize", updatePosition);
});
</script>

<template>
    <div
        ref="triggerRef"
        style="display: contents"
        @mouseenter="showTooltip"
        @mouseleave="hideTooltip"
        @focusin="showTooltip"
        @focusout="hideTooltip"
    >
        <slot />

        <Teleport to="body">
            <Transition name="tooltip-fade">
                <div
                    v-if="isVisible && !disabled"
                    ref="tooltipRef"
                    class="c-tooltip"
                    :style="{ top: `${position.top}px`, left: `${position.left}px` }"
                >
                    {{ text }}
                </div>
            </Transition>
        </Teleport>
    </div>
</template>

<style lang="scss" scoped>
.c-tooltip-trigger {
    display: block;
    width: 100%;
    max-width: 100%;
    min-width: 0;
}

.c-tooltip {
    position: fixed;
    z-index: 10000;
    pointer-events: none;
    background: var(--glass-bg);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    border: 1px solid var(--color-border);
    padding: var(--spacing-xs) var(--spacing-sm);
    border-radius: var(--radius-sm);
    color: var(--color-text);
    font-size: 0.75rem;
    font-weight: 700;
    box-shadow: var(--shadow-md);
    white-space: nowrap;
    max-width: 320px;
    overflow: hidden;
    text-overflow: ellipsis;
}

.tooltip-fade-enter-active,
.tooltip-fade-leave-active {
    transition:
        opacity 0.2s,
        transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.tooltip-fade-enter-from,
.tooltip-fade-leave-to {
    opacity: 0;
    transform: translateY(4px);
}
</style>
