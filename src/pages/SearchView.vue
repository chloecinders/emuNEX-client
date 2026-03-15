<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import GameInfo from "../components/games/GameInfo.vue";
import GameList from "../components/games/GameList.vue";
import Select from "../components/ui/Select.vue";
import { useGameStore, type PartialGame } from "../stores/GameStore";
import { useMetadataStore } from "../stores/MetadataStore";

const gameStore = useGameStore();
const metaStore = useMetadataStore();

const searchQuery = ref("");
const searchResults = ref<PartialGame[]>([]);
const isSearching = ref(false);

const selectedCategory = ref("");
const selectedConsole = ref("");

const overviewGroups = ref<Record<string, PartialGame[]>>({});

const categoryOptions = computed(() => [
    { name: "All Categories", value: "" },
    ...metaStore.categories.map((c) => ({ name: c.name, value: c.name })),
]);

const consoleOptions = computed(() => [
    { name: "All Consoles", value: "" },
    ...metaStore.consoles.map((c) => ({ name: c.name, value: c.name })),
]);

const orderedGroups = computed(() => {
    const keys = Object.keys(overviewGroups.value);
    const result: { title: string; games: PartialGame[] }[] = [];

    if (overviewGroups.value["Most Played"]) {
        result.push({ title: "Most Played", games: overviewGroups.value["Most Played"] });
    }

    if (overviewGroups.value["Recently Added"]) {
        result.push({ title: "Recently Added", games: overviewGroups.value["Recently Added"] });
    }

    const categories = keys
        .filter((k) => k !== "Most Played" && k !== "Recently Added")
        .sort((a, b) => a.localeCompare(b));

    for (const cat of categories) {
        result.push({ title: cat, games: overviewGroups.value[cat] });
    }

    return result;
});

let timeout: ReturnType<typeof setTimeout>;

async function refreshOverview() {
    overviewGroups.value = await gameStore.fetchSearchOverview();
}

onMounted(async () => {
    await Promise.all([metaStore.fetchConsoles(), metaStore.fetchCategories(), refreshOverview()]);
});

async function performSearch() {
    try {
        searchResults.value = await gameStore.searchGames(searchQuery.value, {
            category: selectedCategory.value || null,
            console: selectedConsole.value || null,
        });
    } finally {
        isSearching.value = false;
    }
}

watch([searchQuery, selectedCategory, selectedConsole], () => {
    clearTimeout(timeout);

    if (!searchQuery.value.trim() && !selectedCategory.value && !selectedConsole.value) {
        searchResults.value = [];
        isSearching.value = false;
        return;
    }

    isSearching.value = true;
    searchResults.value = [];
    timeout = setTimeout(performSearch, 400);
});
</script>

<template>
    <Teleport to="#header-tools">
        <div class="c-search-view__bar-container">
            <input v-model="searchQuery" placeholder="Search library..." class="c-search-view__input" autofocus />
        </div>
    </Teleport>

    <div class="c-search-view">
        <div class="c-search-view__filter-bar">
            <div class="c-search-view__filter-item">
                <Select
                    v-model="selectedCategory"
                    :options="categoryOptions"
                    label="Category"
                    placeholder="All Categories"
                />
            </div>

            <div class="c-search-view__filter-item">
                <Select
                    v-model="selectedConsole"
                    :options="consoleOptions"
                    label="Console"
                    placeholder="All Consoles"
                />
            </div>
        </div>

        <div v-if="searchQuery || selectedCategory || selectedConsole" class="c-search-view__results-section">
            <h2 class="c-search-view__group-title">Search Results</h2>

            <div v-if="isSearching" class="c-search-view__loading-overlay">
                <div class="c-search-view__spinner"></div>
            </div>

            <div v-else-if="!searchResults.length" class="c-search-view__prompt">
                <p>No titles found for your filters</p>
            </div>

            <GameList v-else :games="searchResults" />
        </div>

        <div v-else class="c-search-view__overview-section">
            <div v-if="gameStore.loading" class="c-search-view__loading-overlay">
                <div class="c-search-view__spinner"></div>
            </div>
            <template v-else>
                <div v-for="group in orderedGroups" :key="group.title" class="c-search-view__group-block">
                    <h2 class="c-search-view__group-title">{{ group.title }}</h2>
                    <GameList :games="group.games" />
                </div>
            </template>
        </div>
    </div>

    <GameInfo />
</template>

<style lang="scss" scoped>
.c-search-view {
    padding: var(--spacing-md) var(--spacing-lg);

    &__filter-bar {
        display: flex;
        gap: var(--spacing-lg);
        margin-bottom: var(--spacing-xl);
        padding: var(--spacing-lg);
        background: var(--color-surface-variant);
        border-radius: var(--radius-md);
        border: 1px solid var(--color-border);
    }

    &__filter-item {
        flex: 1;
    }

    &__group-block {
        margin-bottom: var(--spacing-xxl);
    }

    &__group-title {
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

    &__bar-container {
        flex: 1;
        max-width: 480px;
        display: flex;
        align-items: center;
    }

    &__input {
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
        box-shadow: var(--shadow-subtle);

        &:focus {
            border-color: var(--color-primary);
            box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.1);
            transform: translateY(-1px);
        }
    }

    &__loading-overlay {
        display: flex;
        justify-content: center;
        padding: var(--spacing-xxl);
    }

    &__spinner {
        width: var(--spacing-xxl);
        height: var(--spacing-xxl);
        border: var(--spacing-xs) solid var(--color-surface-variant);
        border-top-color: var(--color-primary);
        border-radius: var(--radius-full);
        animation: spin 0.8s cubic-bezier(0.4, 0, 0.2, 1) infinite;
    }

    &__prompt {
        text-align: center;
        padding: var(--spacing-xxl);
        color: var(--color-text-muted);
        font-weight: 800;
        font-style: italic;
    }
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}
</style>
