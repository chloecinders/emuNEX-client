<script lang="ts" setup>
import { ref } from "vue";
import { type LibraryGame, useGameStore } from "../../stores/GameStore";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { useStoragePath } from "../../utils/http";
import { onMounted } from "vue";

const gamesStore = useGameStore();
const consoleStore = useConsoleStore();
const localSelectedId = ref<number | null>(null);

onMounted(() => {
    consoleStore.fetchConsoles();
});

defineProps<{
    games: LibraryGame[];
}>();

const changeCurrentGame = (id: number) => {
    localSelectedId.value = id;
    gamesStore.currentSelectedGame = id;
};
</script>

<template>
    <div class="menu-container">
        <div v-if="games.length === 0" class="empty-state"> Your library is empty. </div>

        <div v-else class="game-grid">
            <div
                v-for="game in games"
                :key="game.id"
                class="game-card"
                :class="{
                    'is-selected': localSelectedId === game.id,
                }"
                :style="{ background: consoleStore.getConsoleColor(game.console) }"
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
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: var(--spacing-xl);
    padding: var(--spacing-lg) 0;
}

.game-card {
    position: relative;
    aspect-ratio: 6/9;
    width: 100%;
    background: var(--color-surface);
    border-radius: var(--radius-md);
    overflow: hidden;
    box-shadow: var(--shadow-subtle);
    cursor: pointer;
    transition: all 0.25s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    border: var(--spacing-xs) solid transparent;
}

.game-card:hover {
    transform: scale(1.04) translateY(-var(--spacing-xs));
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
    transform: scale(0.98);
}

.game-card.is-selected {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.2);
    transform: scale(1.04);
}

.empty-state {
    padding: var(--spacing-xxl);
    text-align: center;
    color: var(--color-text-muted);
    font-weight: 800;
    font-style: italic;
}
</style>
