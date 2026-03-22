<script setup lang="ts">
import { Plus, Trash2 } from "lucide-vue-next";
import { computed, nextTick, onMounted, ref } from "vue";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { useGameStore } from "../../stores/GameStore";
import Button from "../ui/Button.vue";
import Heading from "../ui/Heading.vue";
import IconButton from "../ui/IconButton.vue";
import Modal from "../ui/Modal.vue";
import PillButton from "../ui/PillButton.vue";
import Spinner from "../ui/Spinner.vue";
import Text from "../ui/Text.vue";
import GameCard from "./GameCard.vue";
import GameContextMenu from "./GameContextMenu.vue";

const vFocus = {
    mounted: (el: HTMLElement) => nextTick(() => el.focus()),
};

const gameStore = useGameStore();
const consoleStore = useConsoleStore();

const searchQuery = defineModel<string>("searchQuery", { default: "" });
const isCreatingShelf = ref(false);
const newShelfName = ref("");
const editingShelfId = ref<string | null>(null);
const editingShelfName = ref("");
const shelfToDelete = ref<string | null>(null);

onMounted(async () => {
    await Promise.all([consoleStore.fetchConsoles(), gameStore.fetchShelves(), gameStore.fetchLibrary()]);
});

const recentlyPlayedGames = computed(() => {
    return gameStore.library
        .filter((g) => g.last_played)
        .sort((a, b) => new Date(b.last_played!).getTime() - new Date(a.last_played!).getTime())
        .slice(0, 10);
});

const downloadedGames = computed(() => {
    const query = searchQuery.value.trim().toLowerCase();
    const installed = gameStore.library.filter((g) => gameStore.installedGameIds.includes(g.id));

    if (!query) return installed;

    return installed.filter(
        (game) => game.title.toLowerCase().includes(query) || game.console?.toLowerCase().includes(query),
    );
});

const filteredShelves = computed(() => {
    const query = searchQuery.value.trim().toLowerCase();

    let shelves = gameStore.shelves.map((shelf) => ({
        ...shelf,
        games: shelf.games.filter(
            (game) => !query || game.title.toLowerCase().includes(query) || game.console?.toLowerCase().includes(query),
        ),
    }));

    if (query) {
        shelves = shelves.filter((shelf) => shelf.games.length > 0);
    }

    return shelves;
});

const librarySearchResults = computed(() => {
    const query = searchQuery.value.trim().toLowerCase();
    if (!query) return [];

    return gameStore.library.filter(
        (game) => game.title.toLowerCase().includes(query) || game.console?.toLowerCase().includes(query),
    );
});

async function handleCreateShelf() {
    if (!newShelfName.value.trim()) return;
    await gameStore.createShelf(newShelfName.value.trim());
    newShelfName.value = "";
    isCreatingShelf.value = false;
}

function promptDeleteShelf(shelfId: string) {
    shelfToDelete.value = shelfId;
}

async function confirmDeleteShelf() {
    if (shelfToDelete.value !== null) {
        await gameStore.deleteShelf(shelfToDelete.value);
        shelfToDelete.value = null;
    }
}

function startEditingShelf(shelfId: string, currentName: string) {
    editingShelfId.value = shelfId;
    editingShelfName.value = currentName;
}

async function commitShelfRename() {
    if (editingShelfId.value === null) return;
    const trimmed = editingShelfName.value.trim();
    if (trimmed) {
        await gameStore.updateShelf(editingShelfId.value, { name: trimmed });
    }
    editingShelfId.value = null;
    editingShelfName.value = "";
}

const draggedGameId = ref<string | null>(null);
const sourceShelfId = ref<string | null>(null);
const dragGhost = ref<HTMLElement | null>(null);
const isDragging = ref(false);
const hoveredShelfId = ref<string | null>(null);
const insertBeforeGameId = ref<string | null>(null);

const pendingDrag = ref<{ gameId: string; shelfId: string | null; startX: number; startY: number } | null>(null);
const DRAG_THRESHOLD = 5;

function getDragGhostContent(gameId: string): string {
    const game = gameStore.library.find((g) => g.id === gameId);
    if (!game) return "";
    const bg = consoleStore.getConsoleColor(game.console);

    return `<div style="width:80px;height:120px;border-radius:8px;overflow:hidden;background:${bg};box-shadow:0 8px 24px rgba(0,0,0,0.5);transform:rotate(3deg);opacity:0.9;"><img src="http://localhost:1337/storage${game.image_path}" style="width:100%;height:100%;object-fit:cover;" /></div>`;
}

function activateDrag(gameId: string, shelfId: string | null, x: number, y: number) {
    draggedGameId.value = gameId;
    sourceShelfId.value = shelfId;
    isDragging.value = true;

    const ghost = document.createElement("div");
    ghost.id = "drag-ghost";
    ghost.innerHTML = getDragGhostContent(gameId);
    ghost.style.cssText = `
        position: fixed;
        pointer-events: none;
        z-index: 99999;
        left: ${x - 40}px;
        top: ${y - 60}px;
        transition: none;
    `;
    document.body.appendChild(ghost);
    dragGhost.value = ghost;
}

function startDrag(event: MouseEvent, gameId: string, shelfId: string | null) {
    if (event.button !== 0) return;
    event.preventDefault();
    pendingDrag.value = { gameId, shelfId, startX: event.clientX, startY: event.clientY };
    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("mouseup", onMouseUp);
}

function onMouseMove(event: MouseEvent) {
    if (event.buttons === 0) {
        if (isDragging.value || dragGhost.value) {
            if (dragGhost.value) {
                document.body.removeChild(dragGhost.value);
                dragGhost.value = null;
            }
            isDragging.value = false;
            hoveredShelfId.value = null;
            insertBeforeGameId.value = null;
            draggedGameId.value = null;
            sourceShelfId.value = null;
        }
        pendingDrag.value = null;
        window.removeEventListener("mousemove", onMouseMove);
        window.removeEventListener("mouseup", onMouseUp);
        return;
    }

    if (!isDragging.value) {
        if (!pendingDrag.value) return;
        const dx = event.clientX - pendingDrag.value.startX;
        const dy = event.clientY - pendingDrag.value.startY;
        if (Math.sqrt(dx * dx + dy * dy) < DRAG_THRESHOLD) return;

        activateDrag(pendingDrag.value.gameId, pendingDrag.value.shelfId, event.clientX, event.clientY);
        pendingDrag.value = null;
        return;
    }

    if (!dragGhost.value) return;
    dragGhost.value.style.left = `${event.clientX - 40}px`;
    dragGhost.value.style.top = `${event.clientY - 60}px`;

    const el = document.elementFromPoint(event.clientX, event.clientY);
    const shelfEl = el?.closest("[data-shelf-id]");
    const rawShelfId = shelfEl?.getAttribute("data-shelf-id") ?? null;
    if (rawShelfId === "recent") {
        hoveredShelfId.value = "recent";
    } else if (rawShelfId) {
        hoveredShelfId.value = rawShelfId;
    } else {
        hoveredShelfId.value = null;
    }

    const gameEl = el?.closest("[data-game-id]");
    if (gameEl) {
        const gid = gameEl.getAttribute("data-game-id") || null;
        insertBeforeGameId.value = !gid || gid === draggedGameId.value ? null : gid;
    } else {
        insertBeforeGameId.value = null;
    }
}

async function onMouseUp(event: MouseEvent) {
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);

    if (pendingDrag.value) {
        pendingDrag.value = null;
        return;
    }

    if (dragGhost.value) {
        document.body.removeChild(dragGhost.value);
        dragGhost.value = null;
    }

    isDragging.value = false;
    hoveredShelfId.value = null;
    insertBeforeGameId.value = null;

    if (draggedGameId.value === null) return;

    const el = document.elementFromPoint(event.clientX, event.clientY);
    const shelfEl = el?.closest("[data-shelf-id]");
    if (!shelfEl) {
        draggedGameId.value = null;
        sourceShelfId.value = null;
        return;
    }

    const rawTargetId = shelfEl.getAttribute("data-shelf-id");
    const gameId = draggedGameId.value;
    const fromShelfId = sourceShelfId.value;

    if (rawTargetId === "recent" || rawTargetId === "downloaded") {
        if (fromShelfId !== null) {
            await gameStore.removeRomFromShelf(fromShelfId, gameId);
        }
    } else {
        const targetShelfId = rawTargetId || "";
        const targetGameEl = el?.closest("[data-game-id]");
        const targetGameId = targetGameEl ? targetGameEl.getAttribute("data-game-id") || "" : undefined;

        if (targetShelfId) {
            if (fromShelfId === targetShelfId) {
                const shelf = gameStore.shelves.find((s) => s.id === targetShelfId);
                if (shelf && targetGameId && targetGameId !== gameId) {
                    const romIds = shelf.games.map((g) => g.id);
                    const oldIndex = romIds.indexOf(gameId);
                    const newIndex = romIds.indexOf(targetGameId);
                    if (oldIndex !== newIndex && oldIndex !== -1 && newIndex !== -1) {
                        romIds.splice(oldIndex, 1);
                        romIds.splice(newIndex, 0, gameId);
                        await gameStore.updateRomOrder(targetShelfId, romIds);
                    }
                }
            } else {
                if (fromShelfId !== null) {
                    await gameStore.removeRomFromShelf(fromShelfId, gameId);
                }
                await gameStore.addRomToShelf(targetShelfId, gameId);
            }
        }
    }

    draggedGameId.value = null;
    sourceShelfId.value = null;
}

const contextMenu = ref<InstanceType<typeof GameContextMenu> | null>(null);

const onContextMenu = (event: MouseEvent, gameId: string) => {
    contextMenu.value?.open(event, gameId);
};

const onDoubleClickGame = (gameId: string) => {
    gameStore.currentSelectedGame = gameId;
    window.dispatchEvent(new CustomEvent('request-play-game', { detail: { gameId } }));
};
</script>

<template>
    <div class="c-library-wrapper">
        <div class="c-library">
            <Modal :show="isCreatingShelf" title="Create New Shelf" @close="isCreatingShelf = false">
                <div class="c-modal-form">
                    <input
                        v-model="newShelfName"
                        placeholder="Shelf Name..."
                        class="c-input c-input--full"
                        @keyup.enter="handleCreateShelf"
                        autofocus
                    />

                    <div class="c-modal-form__actions">
                        <Button color="grey" @click="isCreatingShelf = false">Cancel</Button>
                        <Button color="primary" @click="handleCreateShelf">Create</Button>
                    </div>
                </div>
            </Modal>

            <Modal :show="shelfToDelete !== null" title="Delete Shelf" @close="shelfToDelete = null">
                <div class="c-modal-form">
                    <Text variant="body" size="md"
                        >Are you sure you want to delete this shelf? Games will not be removed from your library.</Text
                    >

                    <div class="c-modal-form__actions">
                        <Button color="grey" @click="shelfToDelete = null">Cancel</Button>
                        <Button color="red" @click="confirmDeleteShelf">Delete</Button>
                    </div>
                </div>
            </Modal>

            <div v-if="gameStore.loading && !gameStore.shelves.length" class="c-library__loading">
                <Spinner size="lg" />
            </div>

            <div
                v-else-if="!filteredShelves.length && !recentlyPlayedGames.length && !librarySearchResults.length"
                class="c-empty-state"
            >
                <Text v-if="searchQuery" variant="muted" size="lg">No titles found for "{{ searchQuery }}"</Text>
                <Text v-else variant="muted" size="lg">Your library is empty.</Text>
            </div>

            <div v-else class="c-shelves">
                <div
                    v-if="searchQuery ? downloadedGames.length > 0 : true"
                    class="c-shelf"
                    :class="{
                        'c-shelf--drop-target c-shelf--remove-target': isDragging && hoveredShelfId === 'downloaded',
                    }"
                    data-shelf-id="downloaded"
                >
                    <div class="c-shelf__header-wrap">
                        <Heading :level="2" color="primary" is-badge class="c-shelf__badge">
                            Downloaded Games
                            <Text variant="label" size="xs">
                                {{ downloadedGames.length }}
                                {{ downloadedGames.length === 1 ? "title" : "titles" }}
                            </Text>

                            <Text
                                v-for="dummy in [1]"
                                :key="dummy"
                                v-if="isDragging && sourceShelfId !== null"
                                variant="error"
                                size="xs"
                                class="c-shelf__remove-hint"
                            >
                                Drop here to remove from shelf
                            </Text>
                        </Heading>

                        <PillButton @click="isCreatingShelf = true"> <Plus /> New Shelf </PillButton>
                    </div>

                    <div v-if="downloadedGames.length" class="c-shelf__grid">
                        <GameCard
                            v-for="game in downloadedGames"
                            :key="'downloaded-' + game.id"
                            :game="game"
                            :is-dragging="isDragging && draggedGameId === game.id"
                            @mousedown="startDrag($event, game.id, null)"
                            @click="gameStore.currentSelectedGame = game.id"
                            @dblclick="onDoubleClickGame(game.id)"
                            @contextmenu.stop.prevent="onContextMenu($event, game.id)"
                        />
                    </div>
                    <div v-else class="c-shelf__empty-dropzone" style="display: block">
                        <Text variant="muted">No games downloaded yet.</Text>
                    </div>
                </div>

                <div
                    v-if="recentlyPlayedGames.length && !searchQuery"
                    class="c-shelf"
                    :class="{
                        'c-shelf--drop-target c-shelf--remove-target': isDragging && hoveredShelfId === 'recent',
                    }"
                    data-shelf-id="recent"
                >
                    <div class="c-shelf__header-wrap">
                        <Heading level="2" is-badge>
                            Recently Played

                            <Text
                                v-for="dummy in [1]"
                                :key="dummy"
                                v-if="isDragging && sourceShelfId !== null"
                                variant="error"
                                size="xs"
                                class="c-shelf__remove-hint"
                            >
                                Drop here to remove from shelf
                            </Text>
                        </Heading>
                    </div>

                    <div class="c-shelf__grid">
                        <GameCard
                            v-for="game in recentlyPlayedGames"
                            :key="'recent-' + game.id"
                            :game="game"
                            :is-dragging="isDragging && draggedGameId === game.id"
                            @mousedown="startDrag($event, game.id, null)"
                            @click="gameStore.currentSelectedGame = game.id"
                            @dblclick="onDoubleClickGame(game.id)"
                            @contextmenu.stop.prevent="onContextMenu($event, game.id)"
                        />
                    </div>
                </div>

                <div
                    v-for="shelf in filteredShelves"
                    :key="shelf.id"
                    class="c-shelf"
                    :class="{ 'c-shelf--drop-target': isDragging && hoveredShelfId === shelf.id }"
                    :data-shelf-id="shelf.id"
                >
                    <div class="c-shelf__header-wrap">
                        <Heading :level="2" color="primary" is-badge class="c-shelf__badge">
                            <input
                                v-if="editingShelfId === shelf.id"
                                v-model="editingShelfName"
                                class="c-shelf__title-input"
                                @blur="commitShelfRename"
                                @keyup.enter="commitShelfRename"
                                @keyup.escape="editingShelfId = null"
                                v-focus
                            />

                            <Heading
                                v-else
                                :level="2"
                                class="c-shelf__title c-shelf__title--editable"
                                @click="startEditingShelf(shelf.id, shelf.name)"
                                :title="'Click to rename'"
                            >
                                {{ shelf.name }}
                            </Heading>

                            <Text variant="label" size="xs"
                                >{{ shelf.games.length }} {{ shelf.games.length === 1 ? "title" : "titles" }}</Text
                            >
                        </Heading>

                        <IconButton color="red" @click="promptDeleteShelf(shelf.id)" title="Delete Shelf">
                            <Trash2 />
                        </IconButton>
                    </div>

                    <template v-if="shelf.games.length">
                        <div class="c-shelf__grid">
                            <GameCard
                                v-for="game in shelf.games"
                                :key="game.id"
                                :game="game"
                                :is-dragging="isDragging && draggedGameId === game.id"
                                :is-insert-before="
                                    isDragging && insertBeforeGameId === game.id && hoveredShelfId === shelf.id
                                "
                                @mousedown="startDrag($event, game.id, shelf.id)"
                                @click="gameStore.currentSelectedGame = game.id"
                                @dblclick="onDoubleClickGame(game.id)"
                                @contextmenu.stop.prevent="onContextMenu($event, game.id)"
                            />
                        </div>
                    </template>

                    <template v-else>
                        <div class="c-shelf__empty-dropzone"></div>
                    </template>
                </div>

                <div v-if="searchQuery" class="c-shelf">
                    <div class="c-shelf__header-wrap">
                        <Heading :level="2" color="primary" is-badge class="c-shelf__badge">
                            Search Results
                            <Text variant="label" size="xs">
                                {{ librarySearchResults.length }}
                                {{ librarySearchResults.length === 1 ? "match" : "matches" }}
                            </Text>
                        </Heading>
                    </div>

                    <div v-if="librarySearchResults.length" class="c-shelf__grid">
                        <GameCard
                            v-for="game in librarySearchResults"
                            :key="'search-' + game.id"
                            :game="game"
                            @click="gameStore.currentSelectedGame = game.id"
                            @dblclick="onDoubleClickGame(game.id)"
                            @contextmenu.stop.prevent="onContextMenu($event, game.id)"
                        />
                    </div>

                    <div
                        v-else
                        class="c-shelf__empty-dropzone"
                        style="
                            display: block;
                            padding: var(--spacing-xl);
                            text-align: center;
                            border: 2px dashed var(--color-border);
                            border-radius: var(--radius-md);
                            color: var(--color-text-muted);
                        "
                    >
                        No matches found in your library.
                    </div>
                </div>
            </div>
        </div>
    </div>

    <GameContextMenu ref="contextMenu" />
</template>

<style lang="scss" scoped>
.c-library {
    padding: var(--spacing-md) var(--spacing-lg);

    &__loading {
        display: flex;
        justify-content: center;
        padding: var(--spacing-xxl);
    }
}

.c-modal-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);

    &__actions {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-md);
        width: 100%;

        :deep(.c-button) {
            width: 100%;
        }
    }
}

.c-empty-state {
    text-align: center;
    padding: var(--spacing-xxl);
    border-radius: var(--radius-md);
    border: 2px dashed var(--color-border);
}

.c-shelves {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xxl);
}

.c-shelf {
    border-radius: var(--radius-lg);
    padding: var(--spacing-md);
    margin: calc(var(--spacing-md) * -1);
    border: 2px solid transparent;
    transition:
        border-color 0.15s ease,
        background 0.15s ease;

    &--drop-target {
        border-color: var(--color-primary);
        background: color-mix(in srgb, var(--color-primary) 8%, transparent);
    }

    &--remove-target {
        border-color: var(--color-danger);
        background: color-mix(in srgb, var(--color-danger) 8%, transparent);
    }

    &__header-wrap {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-lg);
    }

    &__badge {
        margin-top: calc(var(--spacing-sm) * -1);
    }

    &__title {
        &--editable {
            cursor: text;
            transition: color 0.15s ease;

            &:hover {
                color: var(--color-primary-light);
            }
        }
    }

    &__title-input {
        font-size: 1.5rem;
        font-weight: 800;
        color: var(--color-primary);
        background: transparent;
        border: none;
        border-bottom: 2px solid var(--color-primary);
        outline: none;
        padding: 0;
        min-width: 60px;
        width: auto;
        font-family: inherit;
        line-height: inherit;
        letter-spacing: -0.5px;
    }

    &__remove-hint {
        opacity: 0;
        transition: opacity 0.15s ease;
        white-space: nowrap;

        .c-shelf--remove-target & {
            opacity: 1;
        }
    }

    &__grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: var(--spacing-xl);
    }

    &__empty-dropzone {
        display: none;
    }
}

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

.c-report-notice {
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);

    &--success {
        border-color: color-mix(in srgb, var(--color-success, #22c55e) 45%, var(--color-border));
        background: color-mix(in srgb, var(--color-success, #22c55e) 12%, transparent);
    }

    &--error {
        border-color: color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
        background: color-mix(in srgb, var(--color-danger) 12%, transparent);
    }
}
</style>
