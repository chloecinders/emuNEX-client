<script setup lang="ts">
import { type Ref, ref, watch } from "vue";
import GameInfo from "../components/games/GameInfo.vue";
import GameLibrary from "../components/games/GameLibrary.vue";
import { type LibraryGame, useGameStore } from "../stores/GameStore";

const gameStore = useGameStore();
const library: Ref<LibraryGame[]> = ref([]);

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
    <div class="library-container">
        <h1 class="section-title">Game Library</h1>
        <GameLibrary :games="library" />
    </div>

    <GameInfo />
</template>

<style scoped>
.library-container {
    padding: var(--spacing-md) var(--spacing-lg);
}

.section-title {
    font-size: 1.5rem;
    font-weight: 900;
    color: var(--color-text);
    text-transform: uppercase;
    letter-spacing: -0.5px;
    margin: 0 0 var(--spacing-md) 0;
}
</style>
