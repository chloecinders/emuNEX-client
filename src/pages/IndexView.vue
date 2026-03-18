<script setup lang="ts">
import { type Ref, onMounted, ref, watch } from "vue";
import GameLibrary from "../components/games/GameLibrary.vue";
import SearchSection from "../components/games/SearchSection.vue";
import { type LibraryGame, useGameStore } from "../stores/GameStore";

const gameStore = useGameStore();
const library: Ref<LibraryGame[]> = ref([]);
const searchQuery = ref("");
const ready = ref(false);

onMounted(() => {
    ready.value = true;
});

gameStore.fetchLibrary();

watch(
    () => gameStore.library,
    (newLibrary: LibraryGame[]) => {
        library.value = newLibrary;
    },
    { immediate: true },
);
</script>

<template>
    <div class="c-index-view">
        <Teleport to="#header-tools" v-if="ready">
            <div class="c-search">
                <input v-model="searchQuery" placeholder="Search titles..." class="c-input" autofocus />
            </div>
        </Teleport>

        <GameLibrary v-model:searchQuery="searchQuery" />
        <SearchSection :searchQuery="searchQuery" />
    </div>
</template>

<style lang="scss" scoped>
.c-index-view {
    padding: var(--spacing-md) var(--spacing-lg) var(--spacing-xl) var(--spacing-lg);

    &__title {
        font-size: 1.5rem;
        font-weight: 900;
        color: var(--color-text);
        text-transform: uppercase;
        letter-spacing: -0.5px;
        margin: 0 0 var(--spacing-md) 0;
    }
}

.c-search {
    flex: 1;
    display: flex;
    align-items: center;
    width: 100%;
}

.c-input {
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

    &:focus {
        border-color: var(--color-primary);
        box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.1);
    }
}
</style>
