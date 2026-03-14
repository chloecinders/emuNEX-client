<script lang="ts" setup>
import { ref } from "vue";
import { PartialGame, useGameStore } from "../../stores/GameStore";
import { useMetadataStore } from "../../stores/MetadataStore";
import { useStoragePath } from "../../utils/http";

const gamesStore = useGameStore();
const metadataStore = useMetadataStore();
const localSelectedId = ref<number | null>(null);

defineProps<{
    games: PartialGame[];
}>();

const getConsoleColor = (consoleName: string | undefined) => {
    if (!consoleName) {
        return "#ffffff";
    }

    const consoleData = metadataStore.consoles.find((c) => c.name === consoleName);
    return consoleData?.card_color || "#ffffff";
};

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
                :style="{ backgroundColor: getConsoleColor(game.console) }"
                @click="changeCurrentGame(game.id)"
            >
                <img :src="useStoragePath(game.image_path)" :alt="game.title" />
                <div class="selection-border"></div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.menu-container {
    width: 100%;
    overflow-x: auto;
    scrollbar-width: thin;
}

.game-grid {
    display: flex;
    flex-direction: row;
    gap: 15px;
    padding: 20px;
    align-content: flex-start;
}

.game-card {
    position: relative;
    width: 120px;
    height: 120px;
    border-radius: 12px;
    padding: 8px;
    box-shadow: 0 4px 0 rgba(0, 0, 0, 0.15);
    cursor: pointer;
    transition: transform 0.1s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.game-card:hover {
    transform: scale(1.1);
    z-index: 10;
}

.game-card img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 6px;
    background: white;
}

.game-card:active {
    transform: scale(0.95);
    box-shadow: 0 2px 0 rgba(0, 0, 0, 0.1);
}

.game-card.is-selected {
    outline: 4px solid var(--3ds-blue);
    outline-offset: 2px;
    transform: scale(1.05);
}

.empty-state {
    padding: 40px;
    text-align: center;
    color: #888;
    font-weight: bold;
}
</style>
