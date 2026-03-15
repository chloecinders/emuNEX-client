<script setup lang="ts">
import { Plus, Trash2 } from "lucide-vue-next";
import { computed, nextTick, onMounted, ref } from "vue";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { useGameStore } from "../../stores/GameStore";
import { useStoragePath } from "../../utils/http";
import GameInfo from "../games/GameInfo.vue";
import Button from "../ui/Button.vue";
import Modal from "../ui/Modal.vue";

const vFocus = {
    mounted: (el: HTMLElement) => nextTick(() => el.focus()),
};

const gameStore = useGameStore();
const consoleStore = useConsoleStore();

const searchQuery = ref("");
const isCreatingShelf = ref(false);
const newShelfName = ref("");
const editingShelfId = ref<number | null>(null);
const editingShelfName = ref("");
const shelfToDelete = ref<number | null>(null);

onMounted(async () => {
    await Promise.all([consoleStore.fetchConsoles(), gameStore.fetchShelves(), gameStore.fetchLibrary()]);
});

const recentlyPlayedGames = computed(() => {
    return gameStore.library
        .filter((g) => g.last_played)
        .sort((a, b) => new Date(b.last_played!).getTime() - new Date(a.last_played!).getTime())
        .slice(0, 10);
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

async function handleCreateShelf() {
    if (!newShelfName.value.trim()) return;
    await gameStore.createShelf(newShelfName.value.trim());
    newShelfName.value = "";
    isCreatingShelf.value = false;
}

function promptDeleteShelf(shelfId: number) {
    shelfToDelete.value = shelfId;
}

async function confirmDeleteShelf() {
    if (shelfToDelete.value !== null) {
        await gameStore.deleteShelf(shelfToDelete.value);
        shelfToDelete.value = null;
    }
}

function startEditingShelf(shelfId: number, currentName: string) {
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

const draggedGameId = ref<number | null>(null);
const sourceShelfId = ref<number | null>(null);
const dragGhost = ref<HTMLElement | null>(null);
const isDragging = ref(false);
const hoveredShelfId = ref<number | null>(null);
const insertBeforeGameId = ref<number | null>(null);

const pendingDrag = ref<{ gameId: number; shelfId: number | null; startX: number; startY: number } | null>(null);
const DRAG_THRESHOLD = 5;

function getDragGhostContent(gameId: number): string {
    const game = gameStore.library.find((g) => g.id === gameId);
    if (!game) return "";
    const bg = consoleStore.getConsoleColor(game.console);
    return `<div style="width:80px;height:120px;border-radius:8px;overflow:hidden;background:${bg};box-shadow:0 8px 24px rgba(0,0,0,0.5);transform:rotate(3deg);opacity:0.9;"><img src="${useStoragePath(game.image_path)}" style="width:100%;height:100%;object-fit:cover;" /></div>`;
}

function activateDrag(gameId: number, shelfId: number | null, x: number, y: number) {
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

function startDrag(event: MouseEvent, gameId: number, shelfId: number | null) {
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
        hoveredShelfId.value = -1;
    } else if (rawShelfId) {
        const parsed = parseInt(rawShelfId);
        hoveredShelfId.value = isNaN(parsed) ? null : parsed;
    } else {
        hoveredShelfId.value = null;
    }

    const gameEl = el?.closest("[data-game-id]");
    if (gameEl) {
        const gid = parseInt(gameEl.getAttribute("data-game-id") || "");
        insertBeforeGameId.value = !gid || isNaN(gid) || gid === draggedGameId.value ? null : gid;
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

    if (rawTargetId === "recent") {
        if (fromShelfId !== null) {
            await gameStore.removeRomFromShelf(fromShelfId, gameId);
        }
    } else {
        const targetShelfId = parseInt(rawTargetId || "");
        const targetGameEl = el?.closest("[data-game-id]");
        const targetGameId = targetGameEl ? parseInt(targetGameEl.getAttribute("data-game-id") || "") : undefined;

        if (!isNaN(targetShelfId)) {
            if (fromShelfId === targetShelfId) {
                const shelf = gameStore.shelves.find((s) => s.id === targetShelfId);
                if (shelf && targetGameId && !isNaN(targetGameId) && targetGameId !== gameId) {
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
</script>

<template>
    <Teleport to="#header-tools">
        <div class="search-bar-container">
            <input v-model="searchQuery" placeholder="Search library..." class="nintendo-input" />
        </div>
    </Teleport>

    <div class="library-container">
        <Modal :show="isCreatingShelf" title="Create New Shelf" @close="isCreatingShelf = false">
            <div class="shelf-creator-modal">
                <input
                    v-model="newShelfName"
                    placeholder="Shelf Name..."
                    class="nintendo-input shelf-input"
                    @keyup.enter="handleCreateShelf"
                    autofocus
                />
                <div class="modal-actions">
                    <Button color="grey" @click="isCreatingShelf = false">Cancel</Button>
                    <Button color="blue" @click="handleCreateShelf">Create</Button>
                </div>
            </div>
        </Modal>

        <Modal :show="shelfToDelete !== null" title="Delete Shelf" @close="shelfToDelete = null">
            <div class="shelf-creator-modal">
                <p class="delete-warning"
                    >Are you sure you want to delete this shelf? Games will not be removed from your library.</p
                >
                <div class="modal-actions">
                    <Button color="grey" @click="shelfToDelete = null">Cancel</Button>
                    <Button color="red" @click="confirmDeleteShelf">Delete</Button>
                </div>
            </div>
        </Modal>

        <div v-if="gameStore.loading && !gameStore.shelves.length" class="loading-overlay">
            <div class="spinner"></div>
        </div>

        <div v-else-if="!filteredShelves.length && !recentlyPlayedGames.length" class="empty-state">
            <p v-if="searchQuery">No titles found for "{{ searchQuery }}"</p>
            <p v-else>Your library is empty.</p>
        </div>

        <div v-else class="shelves-list">
            <div v-if="searchQuery" class="shelf-header-container results-header">
                <div class="shelf-header">
                    <h2 class="shelf-title">Search Results</h2>
                </div>
            </div>

            <div
                v-if="recentlyPlayedGames.length && !searchQuery"
                class="shelf-block recently-played"
                :class="{ 'is-drop-target is-remove-target': isDragging && hoveredShelfId === -1 }"
                data-shelf-id="recent"
            >
                <div class="shelf-header-container">
                    <div class="shelf-header">
                        <h2 class="shelf-title">Recently Played</h2>
                        <span v-if="isDragging && sourceShelfId !== null" class="remove-hint"
                            >Drop here to remove from shelf</span
                        >
                    </div>
                    <button class="add-shelf-btn" @click="isCreatingShelf = true">
                        <Plus class="action-icon stroke-primary" /> New Shelf
                    </button>
                </div>

                <div class="shelf-grid">
                    <div
                        v-for="game in recentlyPlayedGames"
                        :key="'recent-' + game.id"
                        class="game-card"
                        :class="{ 'is-dragging': isDragging && draggedGameId === game.id }"
                        :data-game-id="game.id"
                        :style="{ background: consoleStore.getConsoleColor(game.console) }"
                        @mousedown="startDrag($event, game.id, null)"
                        @click="gameStore.currentSelectedGame = game.id"
                    >
                        <img :src="useStoragePath(game.image_path)" :alt="game.title" class="game-cover" />
                    </div>
                </div>
            </div>

            <div
                v-for="shelf in filteredShelves"
                :key="shelf.id"
                class="shelf-block"
                :class="{ 'is-drop-target': isDragging && hoveredShelfId === shelf.id }"
                :data-shelf-id="shelf.id"
            >
                <div class="shelf-header-container">
                    <div class="shelf-header">
                        <input
                            v-if="editingShelfId === shelf.id"
                            v-model="editingShelfName"
                            class="shelf-title-input"
                            @blur="commitShelfRename"
                            @keyup.enter="commitShelfRename"
                            @keyup.escape="editingShelfId = null"
                            v-focus
                        />
                        <h2
                            v-else
                            class="shelf-title shelf-title-editable"
                            @click="startEditingShelf(shelf.id, shelf.name)"
                            :title="'Click to rename'"
                            >{{ shelf.name }}</h2
                        >
                        <span class="game-count">{{ shelf.games.length }} titles</span>
                    </div>
                    <button class="shelf-delete" @click.prevent="promptDeleteShelf(shelf.id)" title="Delete Shelf">
                        <Trash2 class="delete-icon" />
                    </button>
                </div>

                <template v-if="shelf.games.length">
                    <div class="shelf-grid">
                        <div
                            v-for="game in shelf.games"
                            :key="game.id"
                            class="game-card"
                            :class="{
                                'is-dragging': isDragging && draggedGameId === game.id,
                                'insert-before':
                                    isDragging && insertBeforeGameId === game.id && hoveredShelfId === shelf.id,
                            }"
                            :data-game-id="game.id"
                            :style="{ background: consoleStore.getConsoleColor(game.console) }"
                            @mousedown="startDrag($event, game.id, shelf.id)"
                            @click="gameStore.currentSelectedGame = game.id"
                        >
                            <img :src="useStoragePath(game.image_path)" :alt="game.title" class="game-cover" />
                        </div>
                    </div>
                </template>
                <template v-else>
                    <div class="empty-shelf-dropzone"></div>
                </template>
            </div>
        </div>
    </div>

    <GameInfo />
</template>

<style scoped>
.library-container {
    padding: var(--spacing-md) var(--spacing-lg);
}

.library-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-xl);
}

.page-title {
    font-size: 1.1rem;
    font-weight: 800;
    color: var(--color-primary);
    margin: 0;
    display: flex;
    align-items: center;
    background: var(--color-surface-variant);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
}

.add-shelf-btn {
    background: var(--color-surface-variant);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-full);
    padding: 8px 16px;
    font-weight: 800;
    color: var(--color-primary);
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.175, 0.885, 0.32, 1);
    display: flex;
    align-items: center;
    gap: 8px;
}

.action-icon {
    width: 20px;
    height: 20px;
    stroke-width: 2.5px;
}

.add-shelf-btn:hover {
    border-color: var(--color-primary);
    background: var(--color-surface);
    transform: translateY(-2px);
}

.shelf-creator-modal {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
}

.delete-warning {
    margin: 0;
    color: var(--color-text);
    font-size: 1rem;
    font-weight: 700;
}

.shelf-input {
    width: 100%;
}

.modal-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--spacing-md);
    width: 100%;
}

.modal-actions :deep(.nintendo-btn) {
    width: 100%;
}

.shelves-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xxl);
}

.shelf-block {
    animation: fadeIn 0.4s ease-out;
    border-radius: var(--radius-lg);
    padding: var(--spacing-md);
    margin: calc(var(--spacing-md) * -1);
    border: 2px solid transparent;
    transition:
        border-color 0.15s ease,
        background 0.15s ease;
}

.shelf-block.is-drop-target {
    border-color: var(--color-primary);
    background: color-mix(in srgb, var(--color-primary) 8%, transparent);
}

.shelf-block.is-remove-target {
    border-color: var(--color-danger, #e74c3c);
    background: color-mix(in srgb, var(--color-danger, #e74c3c) 8%, transparent);
}

.remove-hint {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-danger, #e74c3c);
    opacity: 0;
    transition: opacity 0.15s ease;
    white-space: nowrap;
}

.is-remove-target .remove-hint {
    opacity: 1;
}

.shelf-title-editable {
    cursor: text;
    transition: color 0.15s ease;
}

.shelf-title-editable:hover {
    color: var(--color-primary);
}

.shelf-title-input {
    font-size: 1.25rem;
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
}

.shelf-header-container {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--spacing-lg);
}

.shelf-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    background: var(--color-surface-variant);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-full);
    width: fit-content;
    border: 1px solid var(--color-border);
}

.shelf-title {
    font-size: 1.1rem;
    font-weight: 800;
    color: var(--color-primary);
    margin: 0;
}

.game-count {
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.shelf-delete {
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 8px;
    border-radius: var(--radius-sm);
    transition: all 0.2s ease;
}

.shelf-delete:hover {
    color: var(--color-danger);
    background: rgba(255, 68, 68, 0.1);
}

.delete-icon {
    width: 20px;
    height: 20px;
    pointer-events: none;
    stroke-width: 2.5px;
}

.shelf-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: var(--spacing-xl);
}

.game-card {
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
    -webkit-user-drag: element;
}

.game-card:hover {
    transform: scale(1.08) translateY(-var(--spacing-xs));
    z-index: 10;
    box-shadow: var(--shadow-md);
    border-color: var(--color-primary-light);
}

.game-card:active {
    transform: scale(0.95);
}

.game-card.is-dragging {
    opacity: 0.35;
    transform: scale(0.95);
}

.game-card.insert-before {
    position: relative;
}

.game-card.insert-before::before {
    content: "";
    position: absolute;
    left: -6px;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--color-primary);
    border-radius: 99px;
    box-shadow: 0 0 8px var(--color-primary);
    z-index: 10;
}

.game-cover {
    width: 100%;
    height: 100%;
    object-fit: cover;
    -webkit-user-drag: none;
}

.game-card * {
    pointer-events: none;
}

.empty-shelf-dropzone {
    display: none;
}

.empty-state {
    text-align: center;
    padding: var(--spacing-xxl);
    color: var(--color-text-muted);
    font-weight: 800;
    font-style: italic;
    border-radius: var(--radius-md);
    border: 2px dashed var(--color-border);
}

.search-bar-container {
    flex: 1;
    max-width: 480px;
    display: flex;
    align-items: center;
}

.nintendo-input {
    width: 100%;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    padding: var(--spacing-sm) var(--spacing-lg);
    font-size: 0.85rem;
    font-weight: 800;
    outline: none;
    transition: all 0.2s cubic-bezier(0.175, 0.885, 0.32, 1);
    color: var(--color-text);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.nintendo-input:focus {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.1);
}

.loading-overlay {
    display: flex;
    justify-content: center;
    padding: var(--spacing-xxl);
}

.spinner {
    width: var(--spacing-xxl);
    height: var(--spacing-xxl);
    border: var(--spacing-xs) solid var(--color-surface-variant);
    border-top: var(--spacing-xs) solid var(--color-primary);
    border-radius: var(--radius-full);
    animation: spin 0.8s cubic-bezier(0.4, 0, 0.2, 1) infinite;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(10px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>
