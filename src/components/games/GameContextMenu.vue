<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useToast } from "../../lib/useToast";
import { useGameStore } from "../../stores/GameStore";
import { useRomStore } from "../../stores/RomStore";
import ReportGameIssue from "../modals/ReportGameIssue.vue";
import Button from "../ui/Button.vue";
import Modal from "../ui/Modal.vue";
import Switch from "../ui/Switch.vue";
import Text from "../ui/Text.vue";

const gameStore = useGameStore();
const romStore = useRomStore();
const toast = useToast();

const show = ref(false);
const x = ref(0);
const y = ref(0);
const gameId = ref<string | null>(null);
const showReportModal = ref(false);

const showDeleteModal = ref(false);
const deleteSaveData = ref(false);

const open = (event: MouseEvent, id: string) => {
    window.dispatchEvent(new CustomEvent("close-all-context-menus"));

    x.value = event.clientX;
    y.value = event.clientY;
    gameId.value = id;
    show.value = true;

    setTimeout(() => {
        const first = document.querySelector<HTMLElement>(".c-context-menu .c-button, .c-context-menu button");
        if (first) {
            first.focus();
        }
    }, 50);

    setTimeout(() => {
        document.addEventListener("click", close, { once: true });
    }, 0);
};

const close = () => {
    show.value = false;
};

const playOrInstall = () => {
    if (!gameId.value) return;
    const id = gameId.value;
    const installed = gameStore.installedGameIds.includes(id);

    gameStore.currentSelectedGame = id;

    if (!installed) {
        window.dispatchEvent(new CustomEvent("request-install-game", { detail: { gameId: id } }));
    } else {
        window.dispatchEvent(new CustomEvent("request-play-game", { detail: { gameId: id } }));
    }

    close();
};

const deleteGame = () => {
    deleteSaveData.value = false;
    showDeleteModal.value = true;
    close();
};

const confirmDeleteGame = async () => {
    if (!gameId.value) return;

    const game = await gameStore.fetchGame(gameId.value);
    if (!game) {
        toast.warning("Could not load game metadata to uninstall.");
        showDeleteModal.value = false;
        return;
    }

    try {
        await romStore.deleteRom(game.id, game.console);
        if (deleteSaveData.value) {
            await romStore.deleteSave(game.id);
        }
        await gameStore.fetchInstalledGames();
    } catch (e) {
        toast.error("Failed to delete game: " + e);
    }

    showDeleteModal.value = false;
};

const openReport = () => {
    showReportModal.value = true;
    close();
};

defineExpose({
    open,
});

const handleCloseAll = () => {
    close();
};

const handleScroll = () => {
    close();
};

onMounted(() => {
    window.addEventListener("close-all-context-menus", handleCloseAll);
    window.addEventListener("scroll", handleScroll, true);
});

onUnmounted(() => {
    window.removeEventListener("close-all-context-menus", handleCloseAll);
    window.removeEventListener("scroll", handleScroll, true);
});

const isInstalled = computed(() => gameStore.installedGameIds.includes(gameId.value || ""));
</script>

<template>
    <div v-if="show" class="c-context-menu" :style="{ left: x + 'px', top: y + 'px' }" @contextmenu.prevent>
        <div class="c-context-menu__primary">
            <Button :color="isInstalled ? 'primary' : 'green'" size="sm" full @click="playOrInstall">
                {{ gameId && isInstalled ? "Play" : "Install" }}
            </Button>
        </div>
        <button v-if="isInstalled" class="c-context-menu__item c-context-menu__item--danger" @click="deleteGame"
            >Delete Game</button
        >
        <button class="c-context-menu__item" @click="openReport">Report Issue...</button>
    </div>

    <ReportGameIssue v-if="gameId" :game-id="gameId" v-model:show-modal="showReportModal" />

    <Modal :show="showDeleteModal" title="Delete Game Data" @close="showDeleteModal = false">
        <div style="display: flex; flex-direction: column; gap: var(--spacing-lg); padding: var(--spacing-sm) 0">
            <Text variant="muted"
                >Are you sure you want to delete this game? This will remove the game files from your system.</Text
            >

            <div
                style="
                    padding: var(--spacing-md);
                    background: var(--color-surface-variant);
                    border-radius: var(--radius-md);
                    border: 1px solid var(--color-border);
                "
            >
                <Switch v-model="deleteSaveData" label="Delete local save data as well" />
                <Text size="xs" variant="muted" style="display: block; margin-top: 4px; padding-left: 2px">
                    Cloud saves will not be affected.
                </Text>
            </div>

            <div
                style="display: flex; gap: var(--spacing-md); justify-content: flex-end; margin-top: var(--spacing-sm)"
            >
                <Button color="grey" @click="showDeleteModal = false">Cancel</Button>
                <Button @click="confirmDeleteGame" style="background: var(--color-red, #ef4444); color: white"
                    >Delete</Button
                >
            </div>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-context-menu {
    position: fixed;
    z-index: 999999;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-lg);
    padding: var(--spacing-xs) 0 var(--spacing-xxs) 0;
    min-width: 140px;

    &__primary {
        padding: var(--spacing-xxs) var(--spacing-xxs) 0 var(--spacing-xxs);

        :deep(.c-button) {
            width: 100%;
        }
    }

    &__item {
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

        &--danger {
            color: var(--color-red, #ef4444);

            &:hover {
                background: color-mix(in srgb, var(--color-red, #ef4444) 15%, transparent);
            }
        }
    }
}
</style>
