<script lang="ts" setup>
defineProps<{
    show: boolean;
    version: number | null;
}>();

const emit = defineEmits(["choice"]);
</script>

<template>
    <transition name="fade">
        <div v-if="show" class="modal-overlay">
            <div class="modal-content">
                <div class="modal-header">
                    <h4>Cloud Sync Conflict</h4>
                </div>
                <div class="modal-body">
                    <p>A newer save (v{{ version }}) exists in the cloud.</p>
                    <p>Would you like to download it and overwrite your local save?</p>
                </div>
                <div class="modal-footer">
                    <button class="btn-3ds btn-cancel" @click="emit('choice', false)">Keep Local</button>
                    <button class="btn-3ds btn-confirm" @click="emit('choice', true)">Download Cloud</button>
                </div>
            </div>
        </div>
    </transition>
</template>

<style scoped>
.modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}
.modal-content {
    background: linear-gradient(to bottom, #ffffff, #e9e9e9);
    border: 3px solid #0089cf;
    border-radius: 15px;
    padding: 20px;
    width: 320px;
    text-align: center;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
}
.modal-body {
    margin: 15px 0;
    font-size: 0.9rem;
    color: #444;
}
.modal-footer {
    display: flex;
    gap: 10px;
    justify-content: center;
}
.btn-cancel {
    background: #888;
    color: white;
    padding: 10px 20px;
}
.btn-confirm {
    background: #0089cf;
    color: white;
    padding: 10px 20px;
}
.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.2s;
}
.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
