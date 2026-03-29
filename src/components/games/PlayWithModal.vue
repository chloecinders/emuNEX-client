<script setup lang="ts">
import type { Emulator } from "../../stores/EmulatorStore";
import type { Game } from "../../stores/GameStore";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import Modal from "../ui/Modal.vue";

defineProps<{
    show: boolean;
    game: Game;
    emulators: Emulator[];
}>();

const emit = defineEmits<{
    (e: "close"): void;
    (e: "play", emulatorId: string): void;
}>();
</script>

<template>
    <Modal :show="show" title="Play with..." @close="emit('close')">
        <div class="c-playwith-list">
            <Button
                v-for="emu in emulators"
                :key="emu.id"
                variant="secondary"
                full
                @click="emit('play', emu.id)"
            >
                {{ emu.name }}
                <Badge v-if="emu.is_default" color="green" style="margin-left: 8px">Default</Badge>
            </Button>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-playwith-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) 0;
}
</style>
