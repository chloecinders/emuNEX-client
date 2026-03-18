<script lang="ts" setup>
import { onMounted, ref } from "vue";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { type PartialGame, useGameStore } from "../../stores/GameStore";
import Button from "../ui/Button.vue";
import GameCard from "./GameCard.vue";
import ReportGameIssue from "../modals/ReportGameIssue.vue";

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

const contextMenu = ref({ show: false, x: 0, y: 0, gameId: null as string | null });

const closeContextMenu = () => {
    contextMenu.value.show = false;
};

const onContextMenu = (event: MouseEvent, gameId: string) => {
    window.dispatchEvent(new CustomEvent("close-all-context-menus"));

    contextMenu.value = {
        show: true,
        x: event.clientX,
        y: event.clientY,
        gameId,
    };
    document.addEventListener("click", closeContextMenu, { once: true });
};

window.addEventListener("close-all-context-menus", () => {
    closeContextMenu();
});

window.addEventListener(
    "scroll",
    () => {
        closeContextMenu();
    },
    true,
);

const playFromContextMenu = () => {
    if (!contextMenu.value.gameId) return;
    const gameId = contextMenu.value.gameId;
    const installed = gamesStore.installedGameIds.includes(gameId);

    gamesStore.currentSelectedGame = gameId;

    if (!installed) {
        window.dispatchEvent(new CustomEvent("request-install-game", { detail: { gameId } }));
    } else {
        window.dispatchEvent(new CustomEvent("request-play-game", { detail: { gameId } }));
    }

    contextMenu.value.show = false;
};

const reportGameModel = ref({ gameId: "", showModal: false });

const submitReport = () => {
    reportGameModel.value.showModal = true;
    reportGameModel.value.gameId = contextMenu.value.gameId || "";
};
</script>

<template>
    <div class="c-game-list" @contextmenu="closeContextMenu">
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

        <div
            v-if="contextMenu.show"
            class="c-game-list__context-menu"
            :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
        >
            <div class="c-game-list__context-primary">
                <Button color="primary" size="sm" full @click="playFromContextMenu">
                    {{
                        contextMenu.gameId && gamesStore.installedGameIds.includes(contextMenu.gameId)
                            ? "Play"
                            : "Install"
                    }}
                </Button>
            </div>
            <button class="c-game-list__context-item" @click="submitReport">Report Issue...</button>
        </div>
    </div>

    <ReportGameIssue :game-id="reportGameModel.gameId" v-model:show-modal="reportGameModel.showModal" />
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
