<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Gamepad2, Library } from "lucide-vue-next";
import { computed, onMounted, ref, type Ref, watch } from "vue";
import { DiscordRPC } from "../../lib/rpc";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { useEmulatorStore } from "../../stores/EmulatorStore";
import { type Game, useGameStore } from "../../stores/GameStore";
import { useStoragePath } from "../../utils/http";
import InstallModal, { type InstallItem } from "../modals/InstallModal.vue";
import SaveConflict from "../modals/SaveConflict.vue";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import Heading from "../ui/Heading.vue";
import IconButton from "../ui/IconButton.vue";
import Modal from "../ui/Modal.vue";
import Text from "../ui/Text.vue";
import Tooltip from "../ui/Tooltip.vue";
import ShelfManager from "./ShelfManager.vue";

const gameStore = useGameStore();
const emulatorStore = useEmulatorStore();
const consoleStore = useConsoleStore();

const game: Ref<Game | null> = ref(null);
const isReadyToPlay = ref(false);
const isDownloading = ref(false);
const showShelfManager = ref(false);
const showConfirmInstallModal = ref(false);
const pendingEmulatorInfo = ref<any>(null);

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

    const emulatorInstalled = await emulatorStore.isEmulatorInstalled(game.value.console);
    const romInstalled = await invoke<boolean>("is_game_installed", {
        gameId: game.value.id.toString(),
        console: game.value.console,
    });

    if (!showConfirmInstallModal.value && (!emulatorInstalled || !romInstalled)) {
        if (!emulatorInstalled) {
            const serverEms = await emulatorStore.fetchServerEmulators(game.value.console);
            pendingEmulatorInfo.value = serverEms[0] || null;

            if (!pendingEmulatorInfo.value) {
                alert(
                    `No emulator found on the server for ${game.value.console.toUpperCase()}. You might need to add one in the Emulator Management page.`,
                );
                return;
            }
        }
        showConfirmInstallModal.value = true;
        return;
    }

    try {
        showConfirmInstallModal.value = false;
        isDownloading.value = true;

        if (!emulatorInstalled) {
            await invoke("download_emulator", {
                console: game.value.console,
                emulatorId: pendingEmulatorInfo.value?.id,
            });
            await emulatorStore.fetchEmulators();
        }

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
        pendingEmulatorInfo.value = null;
    }
};

const installItems = computed<InstallItem[]>(() => {
    if (!game.value) return [];
    const items: InstallItem[] = [
        {
            name: game.value.title,
            description: `Game data for ${game.value.console.toUpperCase()}`,
            size: game.value.file_size_bytes || 0,
            type: "game",
        },
    ];

    if (pendingEmulatorInfo.value) {
        items.push({
            name: pendingEmulatorInfo.value.name,
            description: `Emulator for ${game.value.console.toUpperCase()}`,
            size: pendingEmulatorInfo.value.file_size || 0,
            type: "emulator",
        });
    }

    return items;
});

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

const showEmulatorModal = ref(false);

const handlePlay = async (customEmulatorId?: string) => {
    if (!game.value) return;
    try {
        gameStore.isLaunching = true;
        showEmulatorModal.value = false;
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
        DiscordRPC.playGame(gameIdStr);
        await invoke("install_game", { gameId: gameIdStr, console: game.value.console, romPath: game.value.rom_path });
        await invoke("play_game", {
            gameId: gameIdStr,
            console: game.value.console,
            emulatorId: customEmulatorId || null,
        });
    } catch (error) {
        alert(error);
        gameStore.isLaunching = false;
    } finally {
    }
};
</script>

<template>
    <div class="c-bottom-panel-container">
        <div v-if="game" class="c-bottom-panel">
            <div class="c-bottom-panel__header">
                <div class="c-bottom-panel__banner" :style="{ background: consoleStore.getConsoleColor(game.console) }">
                    <img :src="useStoragePath(game.image_path)" alt="Game Icon" class="c-bottom-panel__thumb" />
                    <Badge
                        class="c-bottom-panel__tag"
                        :bg-color="consoleStore.getConsoleColor(game.console) || 'var(--color-primary)'"
                    >
                        {{ game.console.toUpperCase() }}
                    </Badge>
                </div>

                <div class="c-bottom-panel__titles">
                    <Tooltip :text="game.title">
                        <Heading :level="3" class="c-bottom-panel__title">{{ game.title }}</Heading>
                    </Tooltip>

                    <Text variant="label" size="xs" class="c-bottom-panel__subtitle"
                        >{{ game.category }} | {{ game.region }} | {{ game.release_year }}</Text
                    >

                    <transition name="fade">
                        <div v-if="libraryStats" class="c-bottom-panel__meta">
                            <span class="c-bottom-panel__stat">
                                <span class="c-bottom-panel__stat-label">PLAYED:</span> {{ libraryStats.play_count }}
                            </span>

                            <span class="c-bottom-panel__separator">/</span>

                            <span class="c-bottom-panel__stat">
                                <span class="c-bottom-panel__stat-label">LAST:</span>
                                {{ formatDate(libraryStats.last_played) }}
                            </span>
                        </div>
                    </transition>
                </div>
            </div>

            <div class="c-bottom-panel__action-area">
                <div class="c-bottom-panel__btn-group">
                    <IconButton size="lg" title="Manage Shelves" @click="showShelfManager = true">
                        <Library />
                    </IconButton>

                    <IconButton
                        v-if="isReadyToPlay"
                        size="lg"
                        title="Play with..."
                        @click="showEmulatorModal = true"
                        :disabled="gameStore.isLaunching || gameStore.isPlaying"
                    >
                        <Gamepad2 />
                    </IconButton>

                    <Button
                        v-if="isReadyToPlay"
                        color="blue"
                        full
                        @click="handlePlay()"
                        :disabled="gameStore.isLaunching || gameStore.isPlaying"
                    >
                        <template v-if="gameStore.isLaunching">LAUNCHING</template>
                        <template v-else-if="gameStore.isPlaying">PLAYING</template>
                        <template v-else>PLAY</template>
                    </Button>

                    <Button v-else color="green" full @click="handleInstall" :disabled="isDownloading">
                        {{ isDownloading ? "DOWNLOADING" : "INSTALL" }}
                    </Button>
                </div>
            </div>
        </div>

        <div v-else class="c-bottom-panel c-bottom-panel--loading">
            <Text variant="muted">Loading game data...</Text>
        </div>

        <ShelfManager v-if="game" :game-id="game.id" :show="showShelfManager" @close="showShelfManager = false" />
        <SaveConflict :show="showConflictModal" :version="conflictVersion" @choice="handleConflictChoice" />

        <Modal v-if="game" :show="showEmulatorModal" title="Play with..." @close="showEmulatorModal = false">
            <div class="c-playwith-list">
                <Button
                    v-for="emu in emulatorStore.emulators[game.console.toLowerCase()] || []"
                    :key="emu.id"
                    variant="secondary"
                    full
                    @click="handlePlay(emu.id)"
                >
                    {{ emu.name }}
                    <Badge v-if="emu.is_default" color="green" style="margin-left: 8px">Default</Badge>
                </Button>
            </div>
        </Modal>

        <InstallModal
            v-if="game"
            :show="showConfirmInstallModal"
            :title="`Install ${game.title}`"
            :items="installItems"
            :loading="isDownloading"
            @close="showConfirmInstallModal = false"
            @confirm="handleInstall"
        />
    </div>
</template>

<style lang="scss" scoped>
.c-bottom-panel-container {
    width: 100%;
    display: flex;
    flex-direction: column;
}

.c-bottom-panel {
    position: relative;
    width: 100%;
    max-width: 100vw;
    box-sizing: border-box;
    height: 120px;
    background: transparent;
    padding: var(--spacing-md) var(--spacing-xl);
    display: flex;
    justify-content: space-between;
    align-items: center;
    z-index: 100;
    flex-shrink: 0;
    min-width: 0;
    overflow: visible;

    &__header {
        display: flex;
        gap: var(--spacing-lg);
        align-items: center;
        flex: 1;
        min-width: 0;
        margin-right: var(--spacing-lg);
    }

    &__banner {
        position: relative;
        width: 80px;
        height: 80px;
        flex-shrink: 0;
        background: var(--color-surface);
        border-radius: var(--radius-md);
        padding: var(--spacing-xs);
        box-shadow: var(--shadow-subtle);
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1px solid var(--color-border);
    }

    &__thumb {
        width: 100%;
        height: 100%;
        object-fit: cover;
        border-radius: calc(var(--radius-md) - 4px);
    }

    &__tag {
        position: absolute;
        top: -8px;
        right: -8px;
    }

    &__titles {
        display: flex;
        flex-direction: column;
        flex: 1;
        min-width: 0;
        overflow: hidden;
    }

    &__title {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        width: fit-content;
        max-width: 100%;
        margin: 0;
    }

    &__subtitle {
        margin: var(--spacing-xs) 0 0 0;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    &__meta {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
        font-size: 0.85rem;
        font-weight: 700;
        color: var(--color-text-muted);
    }

    &__stat-label {
        opacity: 0.6;
        margin-right: var(--spacing-xs);
    }

    &__separator {
        opacity: 0.3;
    }

    &__action-area {
        display: flex;
        align-items: center;
        flex-shrink: 0;
    }

    &__btn-group {
        position: relative;
        height: 60px;
        width: 320px;
        flex-shrink: 0;
        display: flex;
        gap: var(--spacing-lg);
    }

    &--loading {
        justify-content: center;
        opacity: 0.5;
    }
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

.c-playwith-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) 0;
}
</style>
