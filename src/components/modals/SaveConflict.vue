<script lang="ts" setup>
import Button from "../ui/Button.vue";

defineProps<{
    show: boolean;
    version: number | null;
}>();

const emit = defineEmits(["choice"]);
</script>

<template>
    <transition name="fade">
        <div v-if="show" class="modal-overlay">
            <div class="modal-card">
                <div class="modal-header">
                    <div class="icon-group">
                        <span class="warning-icon">!</span>
                    </div>
                    <h2 class="title">Sync Conflict</h2>
                    <p class="subtitle">Cloud version v{{ version }} is newer than yours.</p>
                </div>
                
                <div class="modal-body">
                    <p class="description">
                        Would you like to download the newer cloud save and overwrite your local progress? 
                        <strong>This cannot be undone.</strong>
                    </p>
                </div>

                <div class="modal-footer">
                    <Button color="grey" @click="emit('choice', false)">
                        Keep Local
                    </Button>
                    <Button color="blue" @click="emit('choice', true)">
                        Download Cloud
                    </Button>
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
    max-width: 400px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.12);
    border: 1px solid var(--color-border);
    padding: var(--spacing-xxl);
    animation: modal-pop 0.4s cubic-bezier(0.16, 1, 0.3, 1);
    text-align: center;
}

@keyframes modal-pop {
    from { transform: scale(0.95); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
}

.icon-group {
    margin-bottom: var(--spacing-lg);
    display: flex;
    justify-content: center;
}

.warning-icon {
    width: var(--spacing-xxl);
    height: var(--spacing-xxl);
    background: var(--color-primary);
    color: white;
    font-weight: 950;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
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
    font-weight: 600;
}

.modal-body {
    margin: var(--spacing-xl) 0;
}

.description {
    font-size: 0.95rem;
    color: var(--color-text-muted);
    line-height: 1.6;
    font-weight: 500;
}

.description strong {
    color: var(--color-text);
    font-weight: 800;
}

.modal-footer {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
}

.modal-footer :deep(.nintendo-btn) {
    width: 100%;
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
