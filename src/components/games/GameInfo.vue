<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { computed, onMounted, ref, Ref, watch } from "vue";
import { useEmulatorStore } from "../../stores/EmulatorStore";
import { Game, useGameStore } from "../../stores/GameStore";
import { useStoragePath } from "../../utils/http";

const gameStore = useGameStore();
const emulatorStore = useEmulatorStore();

const game: Ref<Game | null> = ref(null);
const isEmulatorInstalled = ref(false);
const isDownloading = ref(false);
const isLaunching = ref(false);

const libraryStats = computed(() => {
    if (!game.value) return null;
    return gameStore.library.find((i) => i.id === game.value?.id);
});

const formatDate = (dateString: string | null) => {
    if (!dateString) return "Never";
    return new Date(dateString).toLocaleDateString();
};

onMounted(async () => {
    await listen("close-prevented", (event) => {
        alert(event.payload);
    });
});

const checkInstallation = async () => {
    if (game.value) {
        isEmulatorInstalled.value = await emulatorStore.isEmulatorInstalled(game.value.console);
    }
};

watch(
    () => gameStore.currentSelectedGame,
    async (newValue) => {
        if (newValue) {
            game.value = await gameStore.fetchGame(newValue);
            await checkInstallation();
        } else {
            game.value = null;
        }
    },
    { immediate: true },
);

const handleInstallEmulator = async () => {
    if (!game.value || isDownloading.value) return;
    try {
        isDownloading.value = true;
        await invoke("download_emulator", { console: game.value.console });
        await emulatorStore.getEmulators();
        await checkInstallation();
    } catch (error) {
        alert(`Failed to download: ${error}`);
    } finally {
        isDownloading.value = false;
    }
};

const handlePlay = async () => {
    if (!game.value || isLaunching.value) return;

    try {
        isLaunching.value = true;
        await gameStore.startGame(game.value.id);

        await invoke("install_game", {
            gameId: game.value.id.toString(),
            console: game.value.console,
            romPath: game.value.rom_path,
        });

        await invoke("play_game", {
            gameId: game.value.id.toString(),
            console: game.value.console,
        });
    } catch (error) {
        alert(error);
    } finally {
        isLaunching.value = false;
    }
};
</script>

<template>
    <transition name="slide-up">
        <div v-if="game" class="bottom-panel">
            <div class="info-header">
                <div class="banner-icon">
                    <img :src="useStoragePath(game.image_path)" alt="Game Icon" class="game-thumb" />
                    <span class="console-tag">{{ game.console.toUpperCase() }}</span>
                </div>
                <div class="titles">
                    <h3>{{ game.title }}</h3>
                    <p class="subtitle">{{ game.category }} | {{ game.region }}</p>

                    <div v-if="libraryStats" class="library-meta">
                        <span>Played {{ libraryStats.play_count }} times</span>
                        <span class="separator">•</span>
                        <span>Last: {{ formatDate(libraryStats.last_played) }}</span>
                    </div>
                </div>
            </div>

            <div class="action-area">
                <button v-if="isEmulatorInstalled" class="btn-3ds btn-play" @click="handlePlay" :disabled="isLaunching">
                    {{ isLaunching ? "Running..." : "Open" }}
                </button>

                <button v-else class="btn-3ds btn-install" @click="handleInstallEmulator" :disabled="isDownloading">
                    {{ isDownloading ? "Downloading..." : "Download Emulator" }}
                </button>
            </div>
        </div>
    </transition>
</template>

<style scoped>
.bottom-panel {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 100px;
    background: linear-gradient(to bottom, #ffffff, #e9e9e9);
    border-top: 3px solid #0089cf;
    padding: 0 30px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    box-shadow: 0 -5px 15px rgba(0, 0, 0, 0.1);
    z-index: 100;
}

.banner-icon {
    position: relative;
    width: 64px;
    height: 64px;
    background: white;
    border-radius: 10px;
    padding: 4px;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
    display: flex;
    align-items: center;
    justify-content: center;
}

.game-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 6px;
}

.console-tag {
    position: absolute;
    bottom: -5px;
    right: -5px;
    background: #444;
    color: white;
    font-size: 8px;
    padding: 2px 5px;
    border-radius: 4px;
    font-weight: bold;
    border: 1px solid white;
}

.info-header {
    display: flex;
    gap: 20px;
    align-items: center;
}

.titles {
    display: flex;
    flex-direction: column;
}

h3 {
    margin: 0;
    font-size: 1.1rem;
    color: #333;
    font-weight: 800;
}

.subtitle {
    margin: 2px 0 0 0;
    font-size: 0.75rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.library-meta {
    margin-top: 4px;
    font-size: 0.7rem;
    color: #0089cf;
    font-weight: bold;
    display: flex;
    gap: 6px;
}

.separator {
    color: #ccc;
}

.btn-3ds {
    padding: 12px 45px;
    border-radius: 50px;
    border: none;
    font-weight: bold;
    font-size: 1rem;
    cursor: pointer;
    box-shadow: 0 4px 0 rgba(0, 0, 0, 0.2);
    transition: all 0.1s;
}

.btn-play {
    background: linear-gradient(to bottom, #82d84a, #4caf50);
    color: white;
}

.btn-install {
    background: linear-gradient(to bottom, #50b6ff, #0089cf);
    color: white;
}

.btn-3ds:active {
    transform: translateY(2px);
    box-shadow: 0 2px 0 rgba(0, 0, 0, 0.2);
}

.slide-up-enter-active,
.slide-up-leave-active {
    transition: transform 0.3s ease-out;
}

.slide-up-enter-from,
.slide-up-leave-to {
    transform: translateY(100%);
}
</style>
