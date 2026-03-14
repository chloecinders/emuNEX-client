<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import GameInfo from "../components/games/GameInfo.vue";
import GameList from "../components/games/GameList.vue";
import { useGameStore, type PartialGame } from "../stores/GameStore";
import { useMetadataStore } from "../stores/MetadataStore";

const gameStore = useGameStore();
const metaStore = useMetadataStore();

const searchQuery = ref("");
const searchResults = ref<PartialGame[]>([]);
const isSearching = ref(false);

const consoleGroups = ref<Record<string, PartialGame[]>>({});
const categoryGroups = ref<Record<string, PartialGame[]>>({});

let timeout: ReturnType<typeof setTimeout>;

onMounted(async () => {
    await Promise.all([metaStore.fetchConsoles(), metaStore.fetchCategories()]);

    for (const con of metaStore.consoles) {
        consoleGroups.value[con.name] = await gameStore.fetchPartialGames({ console: con.name });
    }

    for (const cat of metaStore.categories) {
        categoryGroups.value[cat.name] = await gameStore.fetchPartialGames({ category: cat.name });
    }
});

watch(searchQuery, (newQuery) => {
    clearTimeout(timeout);

    if (!newQuery.trim()) {
        searchResults.value = [];
        isSearching.value = false;
        return;
    }

    isSearching.value = true;

    timeout = setTimeout(async () => {
        try {
            searchResults.value = await gameStore.searchGames(newQuery);
        } finally {
            isSearching.value = false;
        }
    }, 400);
});
</script>

<template>
    <Teleport to="#header-tools">
        <div class="search-bar-container">
            <input v-model="searchQuery" placeholder="Search library..." class="nintendo-input" autofocus />
        </div>
    </Teleport>

    <div class="search-page">
        <div v-if="searchQuery" class="search-results-section">
            <h1 class="section-title">Search Results</h1>

            <div v-if="isSearching" class="loading-overlay">
                <div class="spinner"></div>
            </div>

            <div v-else-if="!searchResults.length" class="search-prompt">
                <p>No titles found for "{{ searchQuery }}"</p>
            </div>

            <GameList v-else :games="searchResults" />
        </div>

        <div v-else class="overview-section">
            <h1 class="section-title">Consoles</h1>

            <div v-for="(games, consoleName) in consoleGroups" :key="consoleName" class="group-block">
                <h2 class="group-title">{{ consoleName }}</h2>
                <GameList :games="games" />
            </div>

            <h1 class="section-title">Categories</h1>

            <div v-for="(games, catName) in categoryGroups" :key="catName" class="group-block">
                <h2 class="group-title">{{ catName }}</h2>
                <GameList :games="games" />
            </div>
        </div>
    </div>

    <GameInfo />
</template>

<style scoped>
.search-page {
    padding: var(--spacing-md) var(--spacing-lg);
}

.section-title {
    font-size: 2.25rem;
    font-weight: 900;
    color: var(--color-text);
    text-transform: uppercase;
    letter-spacing: -1px;
    margin: 0 0 var(--spacing-xl) 0;
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
}

.section-title::after {
    content: "";
    flex: 1;
    height: var(--spacing-xxs);
    background: var(--color-border);
}

.group-block {
    margin-bottom: var(--spacing-xxl);
}

.group-title {
    font-size: 1.1rem;
    font-weight: 800;
    color: var(--color-primary);
    margin-bottom: var(--spacing-lg);
    display: flex;
    align-items: center;
    background: var(--color-surface-variant);
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-full);
    width: fit-content;
    border: 1px solid var(--color-border);
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
    transform: translateY(-1px);
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

.search-prompt {
    text-align: center;
    padding: var(--spacing-xxl);
    color: var(--color-text-muted);
    font-weight: 800;
    font-style: italic;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}
</style>
