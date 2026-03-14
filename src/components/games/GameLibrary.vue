<script lang="ts" setup>
import { ref } from "vue";
import { LibraryGame, useGameStore } from "../../stores/GameStore";
import { useMetadataStore } from "../../stores/MetadataStore";
import { useStoragePath } from "../../utils/http";

const gamesStore = useGameStore();
const metadataStore = useMetadataStore();
const localSelectedId = ref<number | null>(null);

defineProps<{
    games: LibraryGame[];
}>();

await metadataStore.fetchConsoles();

const getConsoleColor = (consoleName: string | undefined) => {
    if (!consoleName) return "#ffffff";
    const consoleData = metadataStore.consoles.find((c) => c.name === consoleName);
    return consoleData?.card_color || "#ffffff";
};

const isDarkColor = (color: string) => {
    const hex = color.replace("#", "");
    const r = parseInt(hex.substr(0, 2), 16);
    const g = parseInt(hex.substr(2, 2), 16);
    const b = parseInt(hex.substr(4, 2), 16);
    const brightness = (r * 299 + g * 587 + b * 114) / 1000;
    return brightness < 128;
};

const changeCurrentGame = (id: number) => {
    localSelectedId.value = id;
    gamesStore.currentSelectedGame = id;
};

const formatDate = (dateString: string | null) => {
    if (!dateString) return "Never";
    return new Date(dateString).toLocaleDateString();
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
                    'is-dark': isDarkColor(getConsoleColor(game.console)),
                }"
                :style="{ backgroundColor: getConsoleColor(game.console) }"
                @click="changeCurrentGame(game.id)"
            >
                <img :src="useStoragePath(game.image_path)" :alt="game.title" />

                <div class="game-stats">
                    <span class="play-count">Plays: {{ game.play_count }}</span>
                    <span class="last-played">{{ formatDate(game.last_played) }}</span>
                </div>
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
    flex-direction: column;
    flex-wrap: wrap;
    height: 375px;
    gap: 15px;
    padding: 20px;
    align-content: flex-start;
}

.game-card {
    position: relative;
    width: 140px;
    height: 160px;
    background: white;
    border-radius: 12px;
    padding: 8px;
    box-shadow: 0 4px 0 rgba(0, 0, 0, 0.15);
    cursor: pointer;
    transition: transform 0.1s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    display: flex;
    flex-direction: column;
}

.game-card:hover {
    transform: scale(1.05);
    z-index: 10;
}

.game-card img {
    width: 100%;
    height: 100px;
    object-fit: cover;
    border-radius: 6px;
    background: white;
}

.game-stats {
    margin-top: 8px;
    display: flex;
    flex-direction: column;
    font-size: 0.7rem;
    color: #444;
    line-height: 1.2;
}

.game-card.is-dark .game-stats {
    color: #eee;
}
.game-card.is-dark .last-played {
    color: #ccc;
}

.play-count {
    font-weight: bold;
}

.last-played {
    color: #888;
}

.game-card:active {
    transform: scale(0.95);
    box-shadow: 0 2px 0 rgba(0, 0, 0, 0.1);
}

.game-card.is-selected {
    outline: 4px solid var(--3ds-blue);
    outline-offset: 2px;
}

.empty-state {
    padding: 40px;
    text-align: center;
    color: #888;
    font-weight: bold;
}
</style>
