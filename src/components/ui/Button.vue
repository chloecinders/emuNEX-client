<script setup lang="ts">
interface Props {
    color?: "blue" | "green" | "red" | "grey";
    disabled?: boolean;
    full?: boolean;
}

withDefaults(defineProps<Props>(), {
    color: "blue",
    disabled: false,
    full: false,
});

const emit = defineEmits(["click"]);
</script>

<template>
    <button
        class="popout-btn"
        :class="[color, { 'is-disabled': disabled, 'is-full': full }]"
        :disabled="disabled"
        @click="emit('click')"
    >
        <span class="btn-shadow"></span>
        <span class="btn-edge"></span>
        <span class="btn-front">
            <slot />
        </span>
    </button>
</template>

<style scoped>
.popout-btn {
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
}

.popout-btn.is-full {
    width: 100%;
    height: 100%;
}

.btn-front {
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    width: 100%;
    padding: 12px 28px;
    border-radius: var(--radius-md);
    color: white;
    transform: translateY(-4px);
    transition: transform 150ms cubic-bezier(0.3, 0.7, 0.4, 1);
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.1);
}

.is-full .btn-front {
    height: 100%;
}

.btn-edge {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    border-radius: var(--radius-md);
}

.blue .btn-front { background: var(--color-primary); }
.blue .btn-edge { background: var(--color-primary-dark); }

.green .btn-front { background: #4caf50; }
.green .btn-edge { background: #3d8b40; }

.red .btn-front { background: #e60012; }
.red .btn-edge { background: #b3000e; }

.grey .btn-front { background: var(--color-text-muted); }
.grey .btn-edge { background: #555; }

.popout-btn:hover .btn-front {
    transform: translateY(-6px);
    filter: brightness(110%);
}

.popout-btn:active .btn-front {
    transform: translateY(-1px);
    transition: transform 34ms;
}

.is-disabled {
    cursor: not-allowed;
    filter: grayscale(0.8);
    opacity: 0.7;
}

.is-disabled .btn-front {
    transform: translateY(-1px) !important;
}
</style>
