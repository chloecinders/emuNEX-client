<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { computed, onMounted, ref, type Ref, watch } from "vue";
import { useEmulatorStore } from "../../stores/EmulatorStore";
import { type Game, useGameStore } from "../../stores/GameStore";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { useStoragePath } from "../../utils/http";
import SaveConflict from "../modals/SaveConflict.vue";
import ShelfManager from "./ShelfManager.vue";
import Button from "../ui/Button.vue";
import { Library } from "lucide-vue-next";

const gameStore = useGameStore();
const emulatorStore = useEmulatorStore();
const consoleStore = useConsoleStore();

const game: Ref<Game | null> = ref(null);
const isReadyToPlay = ref(false);
const isDownloading = ref(false);
const showShelfManager = ref(false);

const libraryStats = computed(() => {
    if (!game.value) return null;
    return gameStore.library.find((i) => i.id === game.value?.id);
});

const formatDate = (dateString: string | null) => {
    if (!dateString) return "Never";
    return new Date(dateString).toLocaleDateString();
};

onMounted(async () => {
    consoleStore.fetchConsoles();
    await listen("close-prevented", (event) => {
        alert(event.payload);
    });
});

const checkInstallation = async () => {
    if (game.value) {
        const emulatorInstalled = await emulatorStore.isEmulatorInstalled(game.value.console);
        const romInstalled = await invoke<boolean>("is_game_installed", {
            gameId: game.value.id.toString(),
            console: game.value.console,
        });

        isReadyToPlay.value = emulatorInstalled && romInstalled;
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

const handleInstall = async () => {
    if (!game.value || isDownloading.value) return;
    try {
        isDownloading.value = true;

        // Check emulator
        const emulatorInstalled = await emulatorStore.isEmulatorInstalled(game.value.console);
        if (!emulatorInstalled) {
            await invoke("download_emulator", { console: game.value.console });
            await emulatorStore.fetchEmulators();
        }

        // Check ROM
        const romInstalled = await invoke<boolean>("is_game_installed", {
            gameId: game.value.id.toString(),
            console: game.value.console,
        });

        if (!romInstalled) {
            await invoke("install_game", {
                gameId: game.value.id.toString(),
                console: game.value.console,
                romPath: game.value.rom_path,
            });
        }

        await checkInstallation();
    } catch (error) {
        alert(`Failed to install: ${error}`);
    } finally {
        isDownloading.value = false;
    }
};

const showConflictModal = ref(false);
const conflictVersion = ref<number | null>(null);
let resolveConflict: ((value: boolean) => void) | null = null;

const promptSyncConflict = (version: number): Promise<boolean> => {
    conflictVersion.value = version;
    showConflictModal.value = true;

    return new Promise((resolve) => {
        resolveConflict = resolve;
    });
};

const handleConflictChoice = (choice: boolean) => {
    showConflictModal.value = false;
    if (resolveConflict) resolveConflict(choice);
};

const handlePlay = async () => {
    if (!game.value) return;
    try {
        gameStore.isLaunching = true;
        const gameIdStr = game.value.id.toString();

        const status: any = await invoke("check_save_status", { gameId: gameIdStr });

        if (status.latest_version !== null) {
            let proceedWithDownload = false;

            if (status.conflict) {
                proceedWithDownload = await promptSyncConflict(status.latest_version);
            } else if (status.latest_version > 0) {
                proceedWithDownload = true;
            }

            if (proceedWithDownload) {
                await invoke("download_save_files", { gameId: gameIdStr });
            }
        }

        await gameStore.startGame(game.value.id);
        await invoke("install_game", { gameId: gameIdStr, console: game.value.console, romPath: game.value.rom_path });
        await invoke("play_game", { gameId: gameIdStr, console: game.value.console });
    } catch (error) {
        alert(error);
        gameStore.isLaunching = false;
    } finally {
        // We don't set isLaunching to false here because the backend event will handle it
    }
};
</script>

<template>
    <transition name="slide-up">
        <div v-if="game" class="bottom-panel">
            <div class="info-header">
                <div class="banner-icon" :style="{ background: consoleStore.getConsoleColor(game.console) }">
                    <img :src="useStoragePath(game.image_path)" alt="Game Icon" class="game-thumb" />
                    <span class="console-tag" :style="{ background: consoleStore.getConsoleColor(game.console) || 'var(--color-primary)' }">{{ game.console.toUpperCase() }}</span>
                </div>

                <div class="titles">
                    <h3>{{ game.title }}</h3>
                    <p class="subtitle">{{ game.category }} | {{ game.region }}</p>

                    <transition name="fade">
                        <div v-if="libraryStats" class="library-meta">
                            <span class="stat-item">
                                <span class="stat-label">PLAYED:</span> {{ libraryStats.play_count }}
                            </span>

                            <span class="separator">/</span>

                            <span class="stat-item">
                                <span class="stat-label">LAST:</span> {{ formatDate(libraryStats.last_played) }}
                            </span>
                        </div>
                    </transition>
                </div>
            </div>

            <div class="action-area">
                <div class="btn-container">
                    <Button
                        v-if="isReadyToPlay"
                        color="blue"
                        full
                        @click="handlePlay"
                        :disabled="gameStore.isLaunching || gameStore.isPlaying"
                    >
                        <template v-if="gameStore.isLaunching">LAUNCHING</template>
                        <template v-else-if="gameStore.isPlaying">PLAYING</template>
                        <template v-else>PLAY</template>
                    </Button>

                    <Button v-else color="green" full @click="handleInstall" :disabled="isDownloading">
                        {{ isDownloading ? "DOWNLOADING" : "INSTALL" }}
                    </Button>

                    <button class="shelf-btn" @click="showShelfManager = true" title="Manage Shelves">
                        <Library class="shelf-icon" />
                    </button>
                </div>
            </div>
        </div>
    </transition>

    <ShelfManager v-if="game" :game-id="game.id" :show="showShelfManager" @close="showShelfManager = false" />
    <SaveConflict :show="showConflictModal" :version="conflictVersion" @choice="handleConflictChoice" />
</template>

<style scoped>
.bottom-panel {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 120px;
    background: var(--glass-bg);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-top: 2px solid var(--color-border);
    padding: 0 var(--spacing-xl);
    display: flex;
    justify-content: space-between;
    align-items: center;
    box-shadow: 0 -10px 30px rgba(0, 0, 0, 0.05);
    z-index: 100;
}

.banner-icon {
    position: relative;
    width: 80px;
    height: 80px;
    background: var(--color-surface);
    border-radius: var(--radius-md);
    padding: var(--spacing-xs);
    box-shadow: var(--shadow-subtle);
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--color-border);
}

.game-thumb {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: calc(var(--radius-md) - 4px);
}

.console-tag {
    position: absolute;
    top: -8px;
    right: -8px;
    background: var(--color-primary);
    color: white;
    font-size: 10px;
    padding: var(--spacing-xxs) var(--spacing-sm);
    border-radius: var(--radius-full);
    font-weight: 900;
    border: 2px solid var(--color-surface);
    box-shadow: var(--shadow-subtle);
}

.info-header {
    display: flex;
    gap: var(--spacing-lg);
    align-items: center;
}

.titles {
    display: flex;
    flex-direction: column;
}

h3 {
    margin: 0;
    font-size: 1.5rem;
    color: var(--color-text);
    font-weight: 900;
    letter-spacing: -0.5px;
}

.subtitle {
    margin: var(--spacing-xs) 0 0 0;
    font-size: 0.8rem;
    color: var(--color-text-muted);
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 1px;
}

.library-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--color-text-muted);
}

.stat-label {
    opacity: 0.6;
    margin-right: var(--spacing-xs);
}

.separator {
    opacity: 0.3;
}

.action-area {
    display: flex;
    align-items: center;
}

.btn-container {
    position: relative;
    height: 60px;
    width: 320px;
    display: flex;
    gap: var(--spacing-lg);
}

.shelf-btn {
    width: 60px;
    height: 60px;
    flex-shrink: 0;
    background: var(--color-surface-variant);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: var(--color-primary);
    transition: all 0.2s cubic-bezier(0.175, 0.885, 0.32, 1);
    box-shadow: 0 4px 10px rgba(0, 0, 0, 0.1);
}

.shelf-btn:hover {
    background: var(--color-surface);
    border-color: var(--color-primary);
    transform: translateY(-4px) scale(1.05);
    box-shadow: 0 8px 20px rgba(107, 92, 177, 0.2);
    color: var(--color-primary-light);
}

.shelf-icon {
    width: 24px;
    height: 24px;
    stroke-width: 2.5px;
}

.slide-up-enter-active,
.slide-up-leave-active {
    transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from,
.slide-up-leave-to {
    transform: translateY(100%);
    opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
