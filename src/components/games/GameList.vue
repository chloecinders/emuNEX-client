<script lang="ts" setup>
import { ref } from "vue";
import { PartialGame, useGameStore } from "../../stores/GameStore";
import { useStoragePath } from "../../utils/http";

const gamesStore = useGameStore();
const localSelectedId = ref<number | null>(null);

defineProps<{
    games: PartialGame[];
}>();

const changeCurrentGame = (id: number) => {
    localSelectedId.value = id;
    gamesStore.currentSelectedGame = id;
};
</script>

<template>
    <div class="menu-container">
        <div v-if="games.length === 0" class="empty-state"> No games found. </div>

        <div v-else class="game-grid">
            <div
                v-for="game in games"
                :key="game.id"
                class="game-card"
                :class="{ 'is-selected': localSelectedId === game.id }"
                @click="changeCurrentGame(game.id)"
            >
                <img :src="useStoragePath(game.image_path)" :alt="game.title" class="game-cover" />
            </div>
        </div>
    </div>
</template>

<style scoped>
.menu-container {
    width: 100%;
}

.game-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: var(--spacing-lg);
    padding: var(--spacing-lg) 0;
}

.game-card {
    position: relative;
    aspect-ratio: 6/9;
    width: 100%;
    border-radius: var(--radius-md);
    overflow: hidden;
    box-shadow: var(--shadow-subtle);
    cursor: pointer;
    transition: all 0.25s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    background: var(--color-surface);
    border: var(--spacing-xxs) solid transparent;
}

.game-card:hover {
    transform: scale(1.08) translateY(-var(--spacing-xs));
    z-index: 10;
    box-shadow: var(--shadow-md);
    border-color: var(--color-primary-light);
}

.game-cover {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.game-card:active {
    transform: scale(0.95);
}

.game-card.is-selected {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.2);
    transform: scale(1.08);
}

.empty-state {
    padding: var(--spacing-xl);
    text-align: center;
    color: var(--color-text-muted);
    font-weight: 800;
    font-style: italic;
}
</style>
