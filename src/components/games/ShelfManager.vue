<script setup lang="ts">
import { computed } from "vue";
import { useGameStore } from "../../stores/GameStore";
import Modal from "../ui/Modal.vue";

const props = defineProps<{
    gameId: string;
    show: boolean;
}>();

const emit = defineEmits(["close"]);

const gameStore = useGameStore();

const shelves = computed(() => gameStore.shelves);

const isGameInShelf = (shelfId: string) => {
    const shelf = gameStore.shelves.find((s) => s.id === shelfId);
    return shelf?.games.some((g) => g.id === props.gameId);
};

async function toggleShelf(shelfId: string) {
    if (isGameInShelf(shelfId)) {
        await gameStore.removeRomFromShelf(shelfId, props.gameId);
    } else {
        await gameStore.addRomToShelf(shelfId, props.gameId);
    }
}
</script>

<template>
    <Modal :show="show" title="Manage Shelves" @close="emit('close')">
        <div class="c-shelf-manager">
            <div v-if="!shelves.length" class="c-shelf-manager__empty">
                No shelves found. Create one in the library!
            </div>
            <div v-for="shelf in shelves" :key="shelf.id" class="c-shelf-manager__item" @click="toggleShelf(shelf.id)">
                <div
                    class="c-shelf-manager__checkbox"
                    :class="{ 'c-shelf-manager__checkbox--checked': isGameInShelf(shelf.id) }"
                >
                    <svg viewBox="0 0 24 24" class="c-shelf-manager__check-icon">
                        <path
                            stroke="currentColor"
                            fill="none"
                            stroke-width="3"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M5 13l4 4L19 7"
                        />
                    </svg>
                </div>
                <span class="c-shelf-manager__name">{{ shelf.name }}</span>
            </div>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-shelf-manager {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);

    &__item {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-md);
        border-radius: var(--radius-md);
        cursor: pointer;
        transition: all 0.2s cubic-bezier(0.175, 0.885, 0.32, 1);
        background: var(--color-surface-variant);
        border: 2px solid transparent;

        &:hover {
            border-color: var(--color-border);
            transform: translateY(-2px);
        }
    }

    &__checkbox {
        width: 24px;
        height: 24px;
        border-radius: 6px;
        border: 2px solid var(--color-border);
        background: var(--color-surface);
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s cubic-bezier(0.175, 0.885, 0.32, 1);

        &--checked {
            background: var(--color-primary);
            border-color: var(--color-primary);
            transform: scale(1.1);

            .c-shelf-manager__check-icon {
                stroke-dashoffset: 0;
            }
        }
    }

    &__check-icon {
        width: 14px;
        height: 14px;
        color: white;
        stroke-dasharray: 24;
        stroke-dashoffset: 24;
        transition: stroke-dashoffset 0.3s cubic-bezier(0.65, 0, 0.45, 1);
    }

    &__name {
        font-weight: 800;
        font-size: 1rem;
        color: var(--color-text);
    }

    &__empty {
        font-size: 0.9rem;
        color: var(--color-text-muted);
        text-align: center;
        padding: var(--spacing-xl);
        font-style: italic;
        font-weight: 700;
    }
}
</style>
