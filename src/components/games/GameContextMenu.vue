<script lang="ts" setup>
import { onMounted, onUnmounted, ref } from "vue";
import { useGameStore } from "../../stores/GameStore";
import Button from "../ui/Button.vue";
import ReportGameIssue from "../modals/ReportGameIssue.vue";

const gameStore = useGameStore();

const show = ref(false);
const x = ref(0);
const y = ref(0);
const gameId = ref<string | null>(null);
const showReportModal = ref(false);

const open = (event: MouseEvent, id: string) => {
    window.dispatchEvent(new CustomEvent("close-all-context-menus"));

    x.value = event.clientX;
    y.value = event.clientY;
    gameId.value = id;
    show.value = true;

    // Close on next click anywhere
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

const openReport = () => {
    showReportModal.value = true;
    close();
};

// Expose open method to parent
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
</script>

<template>
    <div
        v-if="show"
        class="c-context-menu"
        :style="{ left: x + 'px', top: y + 'px' }"
        @contextmenu.prevent
    >
        <div class="c-context-menu__primary">
            <Button color="primary" size="sm" full @click="playOrInstall">
                {{ gameId && gameStore.installedGameIds.includes(gameId) ? "Play" : "Install" }}
            </Button>
        </div>
        <button class="c-context-menu__item" @click="openReport">Report Issue...</button>
    </div>

    <ReportGameIssue
        v-if="gameId"
        :game-id="gameId"
        v-model:show-modal="showReportModal"
    />
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
    }
}
</style>
