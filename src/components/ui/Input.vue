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
    margin-bottom: 20px;
    width: 100%;
}

.input-label {
    display: block;
    font-size: 0.9rem;
    font-weight: bold;
    color: #555;
    margin-bottom: 8px;
    margin-left: 5px;
}

.input-wrapper {
    position: relative;
    border-radius: 8px;
    background: #fff;
    overflow: hidden;
    box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
}

.nintendo-input {
    width: 100%;
    padding: 12px 15px;
    border: 2px solid #ccc;
    border-radius: 8px;
    background: transparent;
    font-size: 1rem;
    color: #333;
    outline: none;
    transition: border-color 0.2s;
    box-sizing: border-box;
}

.nintendo-input:focus {
    border-color: #0089cf;
}

.nintendo-input::placeholder {
    color: #bbb;
}
</style>
