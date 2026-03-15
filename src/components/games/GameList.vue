<script lang="ts" setup>
import { ref } from "vue";
import { type PartialGame, useGameStore } from "../../stores/GameStore";
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
    games: PartialGame[];
}>();

const changeCurrentGame = (id: number) => {
    localSelectedId.value = id;
    gamesStore.currentSelectedGame = id;
};
</script>

<template>
    <div class="c-game-list">
        <div v-if="games.length === 0" class="c-game-list__empty"> No games found. </div>

        <div v-else class="c-game-list__grid">
            <div
                v-for="game in games"
                :key="game.id"
                class="c-game-list__card"
                :class="{ 'c-game-list__card--selected': localSelectedId === game.id }"
                :style="{ background: consoleStore.getConsoleColor(game.console) }"
                @click="changeCurrentGame(game.id)"
            >
                <img :src="useStoragePath(game.image_path)" :alt="game.title" class="c-game-list__cover" />
            </div>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-game-list {
    width: 100%;

    &__grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
        gap: var(--spacing-lg);
        padding: var(--spacing-lg) 0;
    }

    &__card {
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

        &:hover {
            transform: scale(1.08) translateY(-var(--spacing-xs));
            z-index: 10;
            box-shadow: var(--shadow-md);
            border-color: var(--color-primary-light);
        }

        &:active {
            transform: scale(0.95);
        }

        &--selected {
            border-color: var(--color-primary);
            box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.2);
            transform: scale(1.08);
        }
    }

    &__cover {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    &__empty {
        padding: var(--spacing-xl);
        text-align: center;
        color: var(--color-text-muted);
        font-weight: 800;
        font-style: italic;
    }
}
</style>
