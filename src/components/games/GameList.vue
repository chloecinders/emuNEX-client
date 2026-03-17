<script lang="ts" setup>
import { onMounted, ref } from "vue";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { type PartialGame, useGameStore } from "../../stores/GameStore";
import GameCard from "./GameCard.vue";

const gamesStore = useGameStore();
const consoleStore = useConsoleStore();
const localSelectedId = ref<string | null>(null);

onMounted(() => {
    consoleStore.fetchConsoles();
});

defineProps<{
    games: PartialGame[];
}>();

const changeCurrentGame = (id: string) => {
    localSelectedId.value = id;
    gamesStore.currentSelectedGame = id;
};
</script>

<template>
    <div class="c-game-list">
        <div v-if="games.length === 0" class="c-game-list__empty"> No games found. </div>

        <div v-else class="c-game-list__grid">
            <GameCard
                v-for="game in games"
                :key="game.id"
                :game="game"
                :is-selected="localSelectedId === game.id"
                @click="changeCurrentGame(game.id)"
            />
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

    &__empty {
        padding: var(--spacing-xl);
        text-align: center;
        color: var(--color-text-muted);
        font-weight: 800;
        font-style: italic;
    }
}
</style>
