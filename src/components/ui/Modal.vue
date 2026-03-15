<script setup lang="ts">
const props = defineProps<{
    show: boolean;
    title: string;
    subtitle?: string;
}>();

const emit = defineEmits(["close"]);
</script>

<template>
    <transition name="fade">
        <div v-if="show" class="modal-overlay" @click.self="emit('close')">
            <div class="modal-card">
                <button class="modal-close" @click="emit('close')">
                    <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                        <path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
                    </svg>
                </button>

                <div class="modal-header">
                    <h2 class="title">{{ title }}</h2>
                    <p v-if="subtitle" class="subtitle">{{ subtitle }}</p>
                </div>

                <div class="modal-body">
                    <slot></slot>
                </div>
            </div>
        </div>
    </transition>
</template>

<style scoped>
.modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(10, 10, 30, 0.4);
    backdrop-filter: blur(var(--spacing-sm));
    -webkit-backdrop-filter: blur(var(--spacing-sm));
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
}

.modal-card {
    background: var(--color-surface);
    border-radius: var(--radius-md);
    width: 90%;
    max-width: 440px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.12);
    border: 1px solid var(--color-border);
    position: relative;
    padding: var(--spacing-xl);
    animation: modal-pop 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes modal-pop {
    from {
        transform: scale(0.95);
        opacity: 0;
    }
    to {
        transform: scale(1);
        opacity: 1;
    }
}

.modal-close {
    position: absolute;
    top: var(--spacing-md);
    right: var(--spacing-md);
    width: var(--spacing-xl);
    height: var(--spacing-xl);
    border-radius: var(--radius-full);
    border: none;
    background: var(--color-surface-variant);
    color: var(--color-text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
    z-index: 10;
}

.modal-close:hover {
    background: var(--color-primary);
    color: white;
}

.modal-header {
    text-align: center;
    margin-bottom: var(--spacing-xl);
}

.title {
    font-size: 1.5rem;
    font-weight: 950;
    color: var(--color-primary);
    margin: 0;
    letter-spacing: -0.5px;
}

.subtitle {
    margin-top: var(--spacing-sm);
    font-size: 0.85rem;
    color: var(--color-text-muted);
    font-weight: 700;
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.3s ease;
}
.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
