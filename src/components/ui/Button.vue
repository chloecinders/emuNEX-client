<script setup lang="ts">
interface Props {
    color?: "blue" | "green" | "red" | "grey";
    disabled?: boolean;
}

withDefaults(defineProps<Props>(), {
    color: "blue",
    disabled: false,
});

const emit = defineEmits(["click"]);
</script>

<template>
    <button
        class="nintendo-btn"
        :class="[color, { 'is-disabled': disabled }]"
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
.nintendo-btn {
    position: relative;
    border: none;
    background: transparent;
    padding: 0;
    cursor: pointer;
    outline-offset: 4px;
    transition: filter 250ms;
    user-select: none;
    touch-action: manipulation;
    font-family: system-ui, sans-serif;
    font-weight: bold;
    font-size: 1.1rem;
}

/* Base layers */
.btn-front {
    display: block;
    position: relative;
    padding: 10px 35px;
    border-radius: 50px; /* Pill shape */
    color: white;
    transform: translateY(-4px);
    transition: transform 150ms cubic-bezier(0.3, 0.7, 0.4, 1);
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}

.btn-edge {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    border-radius: 50px;
}

.blue .btn-front {
    background: linear-gradient(to bottom, #50b6ff 0%, #0089cf 100%);
}
.blue .btn-edge {
    background: #006da4;
}

.green .btn-front {
    background: linear-gradient(to bottom, #82d84a 0%, #4caf50 100%);
}
.green .btn-edge {
    background: #3d8b40;
}

.red .btn-front {
    background: linear-gradient(to bottom, #ff5f5f 0%, #e60012 100%);
}
.red .btn-edge {
    background: #b3000e;
}

.grey .btn-front {
    background: linear-gradient(to bottom, #bbb 0%, #888 100%);
}
.grey .btn-edge {
    background: #666;
}

.nintendo-btn:hover .btn-front {
    transform: translateY(-6px);
    filter: brightness(110%);
}

.nintendo-btn:active .btn-front {
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
