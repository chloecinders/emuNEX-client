<script lang="ts" setup>
import { onMounted, ref } from "vue";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { type PartialGame, useGameStore } from "../../stores/GameStore";
import GameCard from "./GameCard.vue";
import GameContextMenu from "./GameContextMenu.vue";

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

const contextMenu = ref<InstanceType<typeof GameContextMenu> | null>(null);

const onContextMenu = (event: MouseEvent, gameId: string) => {
    contextMenu.value?.open(event, gameId);
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
                @contextmenu.stop.prevent="
                    changeCurrentGame(game.id);
                    onContextMenu($event, game.id);
                "
            />
        </div>
    </div>

    <GameContextMenu ref="contextMenu" />
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

    &__context-menu {
        position: fixed;
        z-index: 999999;
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-sm);
        box-shadow: var(--shadow-lg);
        padding: var(--spacing-xxs);
        min-width: 180px;
    }

    &__context-primary {
        :deep(.c-button) {
            width: 100%;
        }
    }

    &__context-item {
        display: block;
        width: 100%;
        text-align: left;
        background: transparent;
        border: none;
        padding: var(--spacing-sm) var(--spacing-md);
        color: var(--color-text);
        cursor: pointer;
        font-family: inherit;
        font-size: 0.9rem;

        &:hover {
            background: var(--color-surface-variant);
        }
    }
}
</style>
