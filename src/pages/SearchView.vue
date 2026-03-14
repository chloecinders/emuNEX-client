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
.section-title {
    font-size: 2rem;
    font-weight: 900;
    color: var(--3ds-text);
    border-bottom: 4px solid #eee;
    margin: 0 0 20px 0;
    padding-bottom: 10px;
    text-transform: uppercase;
}

.group-block {
    margin-bottom: 40px;
}

.group-title {
    font-size: 1.2rem;
    font-weight: 800;
    color: var(--3ds-blue);
    margin-bottom: 15px;
    display: flex;
    align-items: center;
}

.group-title::before {
    content: "";
    display: inline-block;
    width: 8px;
    height: 20px;
    background: var(--3ds-blue);
    margin-right: 10px;
    border-radius: 4px;
}

.search-bar-container {
    flex: 1;
    max-width: 400px;
}

.nintendo-input {
    width: 100%;
    background: #fff;
    border: 2px solid #ccc;
    border-radius: 30px;
    padding: 10px 20px;
    font-size: 0.9rem;
    font-weight: bold;
    outline: none;
}

.nintendo-input:focus {
    border-color: var(--3ds-blue);
}

.loading-overlay {
    display: flex;
    justify-content: center;
    padding: 50px;
}

.spinner {
    width: 40px;
    height: 40px;
    border: 4px solid #eee;
    border-top: 4px solid var(--3ds-blue);
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}
</style>
