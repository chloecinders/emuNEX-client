<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ChevronDown, Gamepad2, Library, Loader2 } from "lucide-vue-next";
import { computed, onMounted, ref, type Ref, watch } from "vue";
import { http, useStoragePath } from "../../lib/http";
import { DiscordRPC } from "../../lib/rpc";
import { useToast } from "../../lib/useToast";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { useDownloadStore } from "../../stores/DownloadStore";
import { useEmulatorStore } from "../../stores/EmulatorStore";
import { type Game, useGameStore } from "../../stores/GameStore";
import InstallModal, { type InstallItem } from "../modals/InstallModal.vue";
import MultiDiscDisclaimer from "../modals/MultiDiscDisclaimer.vue";
import SaveConflict from "../modals/SaveConflict.vue";
import Badge from "../ui/Badge.vue";
import Button from "../ui/Button.vue";
import Heading from "../ui/Heading.vue";
import IconButton from "../ui/IconButton.vue";
import Text from "../ui/Text.vue";
import Tooltip from "../ui/Tooltip.vue";
import PlayWithModal from "./PlayWithModal.vue";
import ShelfManager from "./ShelfManager.vue";

const gameStore = useGameStore();
const emulatorStore = useEmulatorStore();
const consoleStore = useConsoleStore();
const downloadStore = useDownloadStore();
const toast = useToast();

const game: Ref<Game | null> = ref(null);
const isReadyToPlay = ref(false);
const isRomInstalled = ref(false);
const isPreparingDownload = ref(false);
const isDownloading = computed(() => {
    if (!game.value) return false;
    return downloadStore.isGameQueued(game.value.id.toString()) || downloadStore.isEmulatorQueued(game.value.console);
});

const activeDownloadItem = computed(() => {
    if (!game.value) return undefined;
    const romItem = downloadStore.getItemForGame(game.value.id.toString());
    const emuItem = downloadStore.getEmulatorItem(game.value.console);

    if (emuItem && emuItem.status === "downloading") return emuItem;
    if (romItem && romItem.status === "downloading") return romItem;
    if (emuItem && emuItem.status === "queued") return emuItem;
    if (romItem && romItem.status === "queued") return romItem;

    return romItem || emuItem;
});

const downloadProgress = computed(() => {
    if (!activeDownloadItem.value || !isDownloading.value) return undefined;
    const item = activeDownloadItem.value;
    if (!item.total_bytes) return 0;
    return Math.min(100, Math.round((item.downloaded_bytes / item.total_bytes) * 100));
});

const extractionProgress = computed(() => {
    if (!activeDownloadItem.value || !isDownloading.value) return undefined;
    return activeDownloadItem.value.extraction_progress;
});

const downloadStatus = computed(() => {
    if (!activeDownloadItem.value || !isDownloading.value) return undefined;
    return activeDownloadItem.value.status;
});

const showShelfManager = ref(false);
const showConfirmInstallModal = ref(false);
const availableEmulators = ref<any[]>([]);
const pendingEmulatorInfo = ref<any>(null);

const showVersionPicker = ref(false);
const loadingVersions = ref(false);
const versions = ref<any[]>([]);
const versionPickerX = ref(0);
const versionPickerY = ref(0);
const splitButtonRef = ref<HTMLElement | null>(null);

const toggleVersionPicker = async () => {
    showVersionPicker.value = !showVersionPicker.value;
    if (showVersionPicker.value) {
        if (splitButtonRef.value) {
            const rect = splitButtonRef.value.getBoundingClientRect();
            versionPickerX.value = rect.right;
            versionPickerY.value = rect.top;
        }

        setTimeout(() => {
            const first = document.querySelector<HTMLElement>(
                ".c-bottom-panel__versions .c-bottom-panel__version-item",
            );
            if (first) first.focus();
        }, 50);

        if (versions.value.length === 0 && game.value) {
            loadingVersions.value = true;

            try {
                versions.value = await gameStore.fetchVersions(game.value.id);
            } catch (err) {
                console.error("[GameInfo] fetchVersions Error:", err);
            } finally {
                loadingVersions.value = false;

                if (showVersionPicker.value) {
                    setTimeout(() => {
                        const first = document.querySelector<HTMLElement>(
                            ".c-bottom-panel__versions .c-bottom-panel__version-item",
                        );
                        if (first) first.focus();
                    }, 50);
                }
            }
        }
    }
};

const selectVersion = (v: any) => {
    gameStore.currentSelectedGame = v.id;
    showVersionPicker.value = false;
};

onMounted(() => {
    window.addEventListener("click", (e) => {
        const target = e.target as HTMLElement;
        if (!target.closest(".c-bottom-panel__split-container") && !target.closest(".c-bottom-panel__versions")) {
            showVersionPicker.value = false;
        }
    });
});

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
        toast.warning(String(event.payload));
    });

    window.addEventListener("request-play-game", (event: Event) => {
        const customEvent = event as CustomEvent<{ gameId: string }>;
        if (!customEvent.detail?.gameId || !game.value || customEvent.detail.gameId !== game.value.id) return;
        handlePlay();
    });

    window.addEventListener("request-install-game", (event: Event) => {
        const customEvent = event as CustomEvent<{ gameId: string }>;
        if (!customEvent.detail?.gameId || !game.value || customEvent.detail.gameId !== game.value.id) return;
        handleInstall();
    });

    window.addEventListener("close-version-picker", () => {
        showVersionPicker.value = false;
    });
});

const checkInstallation = async () => {
    if (game.value) {
        const emulatorInstalled = await emulatorStore.isEmulatorInstalled(game.value.console);
        const romInstalled = await invoke<boolean>("is_game_installed", {
            gameId: game.value.id.toString(),
            console: game.value.console,
        });

        isRomInstalled.value = romInstalled;
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

watch(
    () => gameStore.installedGameIds,
    () => {
        checkInstallation();
    },
    { deep: true },
);

watch(
    () => emulatorStore.emulators,
    () => {
        checkInstallation();
    },
    { deep: true },
);

watch(
    () => game.value?.id,
    () => {
        versions.value = [];
        showVersionPicker.value = false;
    },
);

const formatError = (err: any): string => {
    if (!err) return "Unknown error";
    if (typeof err === "string") return err;
    if (err.message) return err.message;
    try {
        return JSON.stringify(err, null, 2);
    } catch {
        return String(err);
    }
};

const handleInstall = async () => {
    if (!game.value || isDownloading.value) return;

    try {
        isPreparingDownload.value = true;
        const emulatorInstalled = await emulatorStore.isEmulatorInstalled(game.value.console);
        const romInstalled = await invoke<boolean>("is_game_installed", {
            gameId: game.value.id.toString(),
            console: game.value.console,
        });

        if (!showConfirmInstallModal.value) {
            const allServerEms = await emulatorStore.fetchAllServerEmulators();

            const serverEms = allServerEms.filter((se) =>
                (se.consoles || []).some((c) => c.toLowerCase() === game.value!.console.toLowerCase()),
            );

            const uniqueServerEmsMap = new Map();
            for (const se of serverEms) {
                uniqueServerEmsMap.set(se.id, se);
            }
            const uniqueServerEms = Array.from(uniqueServerEmsMap.values());

            const localEms = Object.values(emulatorStore.emulators).filter((e) =>
                (e.consoles || []).some((c) => c.toLowerCase() === game.value!.console.toLowerCase()),
            );

            const combinedOptions = [];

            for (const le of localEms) {
                combinedOptions.push({
                    id: le.id,
                    name: le.name,
                    file_size: 0,
                    is_local: true,
                    is_default: le.is_default,
                });
            }

            for (const se of uniqueServerEms) {
                const isInstalled = localEms.some((le) => String(le.id).endsWith(String(se.id)));
                if (!isInstalled) {
                    combinedOptions.push({
                        ...se,
                        id: String(se.id),
                        is_local: false,
                        is_default: false,
                    });
                }
            }

            availableEmulators.value = combinedOptions;

            if (combinedOptions.length > 0) {
                const defaultEmu =
                    combinedOptions.find((e) => e.is_default) ||
                    combinedOptions.find((e) => e.is_local) ||
                    combinedOptions[0];
                pendingEmulatorInfo.value = defaultEmu;
            } else {
                pendingEmulatorInfo.value = null;
            }

            if (!pendingEmulatorInfo.value && !emulatorInstalled) {
                toast.error(
                    `No emulator found for ${game.value.console.toUpperCase()}. You might need to add one in the Emulator Management page.`,
                );
                return;
            }

            showConfirmInstallModal.value = true;
            isPreparingDownload.value = false;
            return;
        }

        showConfirmInstallModal.value = false;

        if (pendingEmulatorInfo.value && !pendingEmulatorInfo.value.is_local) {
            await downloadStore.enqueueEmulator({
                label: `${pendingEmulatorInfo.value.name} (${game.value.console.toUpperCase()})`,
                console: game.value.console,
                emulator_id: pendingEmulatorInfo.value.id,
                total_bytes: pendingEmulatorInfo.value.file_size || 0,
            });
        } else if (pendingEmulatorInfo.value && pendingEmulatorInfo.value.is_local) {
            await emulatorStore.setDefaultEmulator(pendingEmulatorInfo.value.id);
        }

        if (!romInstalled) {
            const dlRes = await http.post<any>(`/roms/${game.value.id}/download`, {});

            if (!dlRes.success) {
                throw new Error(dlRes.message || "Failed to get ROM path from server");
            }

            let romPath = "";
            let zipped = false;
            let zippedEntry = null;

            if (typeof dlRes.data === "string") {
                romPath = dlRes.data;
            } else {
                romPath = dlRes.data.rom_path;
                zipped = dlRes.data.zipped || false;
                zippedEntry = dlRes.data.zipped_entry || null;
            }

            await downloadStore.enqueueRom({
                label: game.value.realname || game.value.title,
                game_id: game.value.id.toString(),
                console: game.value.console,
                rom_path: romPath,
                extension: game.value.file_extension || "rom",
                name: game.value.realname || game.value.title,
                zipped: zipped,
                zipped_entry: zippedEntry,
                total_bytes: game.value.file_size_bytes || 0,
                md5: dlRes.data?.md5 || undefined,
            });
        }
        pendingEmulatorInfo.value = null;
        availableEmulators.value = [];
        isPreparingDownload.value = false;
    } catch (error) {
        toast.error(`Failed to queue install: ${formatError(error)}`);
        isPreparingDownload.value = false;
    }
};

const handleCancelInstall = () => {
    showConfirmInstallModal.value = false;
    pendingEmulatorInfo.value = null;
    availableEmulators.value = [];
};

const installItems = computed<InstallItem[]>(() => {
    if (!game.value) return [];
    const items: InstallItem[] = [];

    if (!isRomInstalled.value) {
        items.push({
            name: game.value.title,
            description: `Game data for ${game.value.console.toUpperCase()}`,
            size: game.value.file_size_bytes || 0,
            type: "game",
        });
    }

    if (pendingEmulatorInfo.value && !pendingEmulatorInfo.value.is_local) {
        items.push({
            name: pendingEmulatorInfo.value.name,
            description: `Emulator for ${game.value.console.toUpperCase()}`,
            size: pendingEmulatorInfo.value.file_size || 0,
            type: "emulator",
        });
    }

    return items;
});

const formatBytes = (bytes: number, decimals = 2) => {
    if (!+bytes) return "0 Bytes";
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
};

const emulatorOptions = computed(() => {
    const options = availableEmulators.value.map((emu) => {
        let domain = "Local";
        if (emu.source_server) {
            domain = emu.source_server.replace(/^https?:\/\//, "").replace(/\/$/, "");
        } else if (String(emu.id).startsWith("server-")) {
            const idParts = String(emu.id).split("-");
            if (idParts.length >= 3) {
                const domainEncoded = idParts[1];
                domain = domainEncoded
                    .replace(/___/g, "://")
                    .replace(/_/g, ".")
                    .replace(/^https?:\/\//, "");
            }
        }

        let desc = "";
        if (emu.is_local) {
            desc = `${domain} (Installed)`;
        } else {
            desc = `${domain} - ${formatBytes(emu.file_size)}`;
        }

        return {
            value: String(emu.id),
            name: emu.name,
            description: desc,
        };
    });
    return options;
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

const showDisclaimerModal = ref(false);
let resolveDisclaimer: ((value: boolean) => void) | null = null;

const promptDisclaimer = (): Promise<boolean> => {
    showDisclaimerModal.value = true;
    return new Promise((resolve) => {
        resolveDisclaimer = resolve;
    });
};

const handleDisclaimerChoice = (choice: boolean) => {
    showDisclaimerModal.value = false;
    if (resolveDisclaimer) resolveDisclaimer(choice);
};

const showEmulatorModal = ref(false);

const handlePlay = async (customEmulatorId?: string) => {
    if (!game.value) return;

    if (!isReadyToPlay.value) {
        handleInstall();
        return;
    }

    try {
        gameStore.isLaunching = true;
        showEmulatorModal.value = false;
        const gameIdStr = game.value.id.toString();

        if (game.value.multi_disc_disclaimer && !gameStore.hideMultiDiscDisclaimer) {
            const proceed = await promptDisclaimer();
            if (!proceed) {
                gameStore.isLaunching = false;
                return;
            }
        }

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

        await invoke("play_game", {
            gameId: gameIdStr,
            console: game.value.console,
            emulatorId: customEmulatorId || null,
        });
    } catch (error) {
        toast.error(`Launch Failed: ${formatError(error)}`);
        gameStore.isLaunching = false;
    } finally {
        gameStore.isLaunching = false;
    }
};
</script>

<template>
    <div class="c-bottom-panel-container">
        <div v-if="game" class="c-bottom-panel">
            <div class="c-bottom-panel__header">
                <div class="c-bottom-panel__banner">
                    <img
                        v-if="game"
                        :src="useStoragePath(game.image_path)"
                        alt="Game Icon"
                        class="c-bottom-panel__thumb" />
                </div>

                <div class="c-bottom-panel__titles">
                    <Tooltip :text="game.realname || game.title">
                        <Heading :level="3" class="c-bottom-panel__title">{{ game.title }}</Heading>
                    </Tooltip>

                    <div class="c-bottom-panel__tags">
                        <Badge :bg-color="consoleStore.getConsoleColor(game.console) || 'var(--color-primary)'">
                            {{ game.console.toUpperCase() }}
                        </Badge>

                        <span v-if="game.category" class="c-bottom-panel__dot"></span>
                        <Text v-if="game.category" variant="label" size="xs" class="c-bottom-panel__tag-text">
                            {{ game.category }}
                        </Text>

                        <span v-if="game.region" class="c-bottom-panel__dot"></span>
                        <Text v-if="game.region" variant="label" size="xs" class="c-bottom-panel__tag-text">
                            {{ game.region }}
                        </Text>

                        <span v-if="game.release_year" class="c-bottom-panel__dot"></span>
                        <Text v-if="game.release_year" variant="label" size="xs" class="c-bottom-panel__tag-text">
                            {{ game.release_year }}
                        </Text>
                    </div>

                    <transition name="fade">
                        <div v-if="libraryStats" class="c-bottom-panel__meta">
                            <span class="c-bottom-panel__stat">
                                <span class="c-bottom-panel__stat-label">PLAYED</span>
                                <span class="c-bottom-panel__stat-value">{{ libraryStats.play_count }}</span>
                            </span>

                            <span class="c-bottom-panel__stat">
                                <span class="c-bottom-panel__stat-label">LAST</span>
                                <span class="c-bottom-panel__stat-value">
                                    {{ formatDate(libraryStats.last_played) }}
                                </span>
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
                        :disabled="gameStore.isLaunching || gameStore.isPlaying">
                        <Gamepad2 />
                    </IconButton>

                    <div
                        v-if="!game.versions_count || game.versions_count <= 1"
                        class="c-bottom-panel__split-container">
                        <Button
                            v-if="isReadyToPlay"
                            color="primary"
                            full
                            @click="handlePlay()"
                            :disabled="gameStore.isLaunching || gameStore.isPlaying">
                            <template v-if="gameStore.isLaunching">LAUNCHING</template>
                            <template v-else-if="gameStore.isPlaying">PLAYING</template>
                            <template v-else>PLAY</template>
                        </Button>
                        <Button
                            v-else
                            color="green"
                            full
                            @click="handleInstall"
                            :disabled="isDownloading || isPreparingDownload"
                            :progress="downloadProgress === 100 ? extractionProgress : downloadProgress">
                            <template v-if="isDownloading">
                                {{
                                    downloadStatus === "queued"
                                        ? "QUEUED"
                                        : downloadProgress === 100
                                          ? game?.zipped
                                              ? `UNPACKING ${extractionProgress ?? 0}%`
                                              : `INSTALLING ${extractionProgress ?? 0}%`
                                          : `DOWNLOADING ${downloadProgress ?? 0}%`
                                }}
                            </template>
                            <template v-else-if="isPreparingDownload">
                                <Loader2 class="spin" :size="18" />
                                PREPARING...
                            </template>
                            <template v-else>INSTALL</template>
                        </Button>
                    </div>

                    <div v-else class="c-bottom-panel__split-container">
                        <div class="c-bottom-panel__split" ref="splitButtonRef">
                            <Button
                                v-if="isReadyToPlay"
                                color="primary"
                                class="c-bottom-panel__split-main"
                                @click="handlePlay()"
                                :disabled="gameStore.isLaunching || gameStore.isPlaying">
                                <template v-if="gameStore.isLaunching">LAUNCHING</template>
                                <template v-else-if="gameStore.isPlaying">PLAYING</template>
                                <template v-else>PLAY</template>
                            </Button>
                            <Button
                                v-else
                                color="green"
                                class="c-bottom-panel__split-main"
                                @click="handleInstall"
                                :disabled="isDownloading || isPreparingDownload"
                                :progress="downloadProgress === 100 ? extractionProgress : downloadProgress">
                                <template v-if="isDownloading">
                                    {{
                                        downloadStatus === "queued"
                                            ? "QUEUED"
                                            : downloadProgress === 100
                                              ? game?.zipped
                                                  ? `UNPACKING ${extractionProgress ?? 0}%`
                                                  : `INSTALLING ${extractionProgress ?? 0}%`
                                              : `DOWNLOADING ${downloadProgress ?? 0}%`
                                    }}
                                </template>
                                <template v-else-if="isPreparingDownload">
                                    <Loader2 class="spin" :size="18" />
                                    PREPARING...
                                </template>
                                <template v-else>INSTALL</template>
                            </Button>

                            <Button
                                class="c-bottom-panel__split-arrow"
                                :color="isReadyToPlay ? 'primary' : 'green'"
                                :class="{
                                    'c-bottom-panel__split-arrow--open': showVersionPicker,
                                    'is-ready': isReadyToPlay,
                                }"
                                @click="toggleVersionPicker"
                                title="Choose version">
                                <ChevronDown :size="18" />
                            </Button>
                        </div>

                        <Teleport to="body">
                            <div
                                v-if="showVersionPicker"
                                class="c-bottom-panel__versions"
                                :style="{
                                    position: 'fixed',
                                    right: `calc(100vw - ${versionPickerX}px)`,
                                    bottom: `calc(100vh - ${versionPickerY}px + 8px)`,
                                }">
                                <div class="c-bottom-panel__versions-header">{{ game.versions_count }} VERSIONS</div>
                                <div v-if="loadingVersions" class="c-bottom-panel__versions-loading">
                                    Loading versions…
                                </div>
                                <template v-else>
                                    <button
                                        v-for="v in versions"
                                        :key="v?.id || Math.random()"
                                        v-show="v"
                                        class="c-bottom-panel__version-item"
                                        @click="selectVersion(v)"
                                        :class="{ 'is-active': v?.id === game.id }">
                                        <img
                                            v-if="v?.image_path"
                                            :src="useStoragePath(v.image_path)"
                                            class="c-bottom-panel__version-thumb"
                                            :alt="v.title" />
                                        <div class="c-bottom-panel__version-info" v-if="v">
                                            <span class="c-bottom-panel__version-name">
                                                {{ v.realname || v.region || v.title }}
                                            </span>
                                            <span v-if="v.region" class="c-bottom-panel__version-region">
                                                {{ v.region }}
                                            </span>
                                        </div>
                                        <div
                                            v-if="v && gameStore.installedGameIds.includes(v.id)"
                                            class="c-bottom-panel__version-installed">
                                            installed
                                        </div>
                                    </button>
                                </template>
                            </div>
                        </Teleport>
                    </div>
                </div>
            </div>
        </div>

        <div v-else class="c-bottom-panel c-bottom-panel--loading">
            <Text variant="muted">Loading game data...</Text>
        </div>

        <ShelfManager v-if="game" :game-id="game.id" :show="showShelfManager" @close="showShelfManager = false" />
        <SaveConflict :show="showConflictModal" :version="conflictVersion" @choice="handleConflictChoice" />
        <MultiDiscDisclaimer
            :show="showDisclaimerModal"
            @confirm="handleDisclaimerChoice(true)"
            @cancel="handleDisclaimerChoice(false)" />

        <PlayWithModal
            v-if="game"
            :show="showEmulatorModal"
            :game="game"
            :emulators="
                Object.values(emulatorStore.emulators).filter((e) => e.consoles.some((c) => c === game!.console))
            "
            @close="showEmulatorModal = false"
            @play="handlePlay" />

        <InstallModal
            v-if="game"
            :show="showConfirmInstallModal"
            :title="`Install ${game.title}`"
            :items="installItems"
            :loading="isDownloading || isPreparingDownload"
            :emulatorOptions="emulatorOptions"
            :selectedEmulatorId="pendingEmulatorInfo?.id || ''"
            @update:selectedEmulatorId="
                (val) => {
                    pendingEmulatorInfo = availableEmulators.find((em) => String(em.id) === String(val));
                }
            "
            @close="handleCancelInstall"
            @confirm="handleInstall" />
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
        width: 88px;
        height: 88px;
        flex-shrink: 0;
        background: var(--color-surface-hover);
        border-radius: var(--radius-lg);
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    &__thumb {
        width: 100%;
        height: 100%;
        object-fit: cover;
        border-radius: calc(var(--radius-lg) - 1px);
    }

    &__titles {
        display: flex;
        flex-direction: column;
        justify-content: center;
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
        line-height: 1.2;
    }

    &__tags {
        display: flex;
        align-items: center;
        gap: 8px;
        margin: 6px 0 10px 0;
        flex-wrap: nowrap;
        overflow: hidden;
    }

    &__dot {
        width: 4px;
        height: 4px;
        border-radius: 50%;
        background: var(--color-text-muted);
        opacity: 0.5;
        flex-shrink: 0;
    }

    &__tag-text {
        color: var(--color-text-muted);
        white-space: nowrap;
        flex-shrink: 0;
    }

    &__meta {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    &__stat {
        display: flex;
        align-items: baseline;
        gap: 6px;
        background: rgba(255, 255, 255, 0.04);
        padding: 4px 10px;
        border-radius: var(--radius-sm);
        border: 1px solid rgba(255, 255, 255, 0.05);
    }

    &__stat-label {
        font-size: 0.7rem;
        font-weight: 800;
        color: var(--color-text-muted);
        letter-spacing: 0.5px;
    }

    &__stat-value {
        font-size: 0.85rem;
        font-weight: 700;
        color: var(--color-text);
    }

    &__action-area {
        display: flex;
        align-items: center;
        flex-shrink: 0;
    }

    &__btn-group {
        position: relative;
        height: 60px;
        width: 340px;
        flex-shrink: 0;
        display: flex;
        gap: var(--spacing-lg);
    }

    &__split-container {
        position: relative;
        flex: 1;
        min-width: 0;
    }

    &__split {
        display: flex;
        height: 100%;
        gap: 2px;
        align-items: stretch;
    }

    &__split-main {
        flex: 1;
        :deep(.c-button) {
            border-top-right-radius: 0;
            border-bottom-right-radius: 0;
        }
        :deep(.c-button__front),
        :deep(.c-button__edge) {
            border-top-right-radius: 0;
            border-bottom-right-radius: 0;
        }
    }

    &__split-arrow {
        width: 40px;
        flex-shrink: 0;

        :deep(.c-button) {
            border-top-left-radius: 0;
            border-bottom-left-radius: 0;
        }
        :deep(.c-button__front),
        :deep(.c-button__edge) {
            border-top-left-radius: 0;
            border-bottom-left-radius: 0;
        }

        :deep(.c-button__front) {
            padding: 0;
            border-left: 1px solid rgba(0, 0, 0, 0.2);
            justify-content: center;
        }

        &--open :deep(.c-button__front) {
            transform: translateY(-1px);
            filter: brightness(90%);
        }

        svg {
            transition: transform 0.2s ease;
        }
        &--open svg {
            transform: rotate(180deg);
        }
    }

    &__versions {
        position: fixed;
        min-width: 300px;
        width: max-content;
        max-width: 500px;
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        box-shadow: var(--shadow-xl);
        overflow: hidden;
        max-height: 250px;
        overflow-y: auto;
        z-index: 1000;
    }

    &__versions-header {
        padding: var(--spacing-sm) var(--spacing-md);
        font-size: 0.75rem;
        font-weight: 800;
        color: var(--color-text-muted);
        text-transform: uppercase;
        border-bottom: 1px solid var(--color-border);
        background: color-mix(in srgb, var(--color-surface-variant) 50%, transparent);
    }

    &__versions-loading {
        padding: var(--spacing-md);
        color: var(--color-text-muted);
        text-align: center;
        font-size: 0.9rem;
    }

    &__version-item {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        width: 100%;
        background: transparent;
        border: none;
        border-bottom: 1px solid var(--color-border);
        padding: var(--spacing-sm) var(--spacing-md);
        cursor: pointer;
        text-align: left;
        font-family: inherit;
        transition: background 0.15s;

        &:last-child {
            border-bottom: none;
        }

        &:hover,
        &:focus {
            background: var(--color-surface-variant);
            outline: none;
        }

        &.is-active {
            background: color-mix(in srgb, var(--color-primary) 10%, transparent);
            &:focus {
                background: color-mix(in srgb, var(--color-primary) 20%, transparent);
            }
        }
    }

    &__version-thumb {
        width: 32px;
        height: 32px;
        border-radius: var(--radius-xs, 4px);
        object-fit: cover;
        flex-shrink: 0;
        border: 1px solid var(--color-border);
    }

    &__version-info {
        display: flex;
        flex-direction: column;
        flex: 1;
        min-width: 0;
    }

    &__version-name {
        font-size: 0.85rem;
        font-weight: 600;
        color: var(--color-text);
        white-space: normal;
        overflow: visible;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        line-clamp: 2;
    }

    &__version-region {
        font-size: 0.7rem;
        color: var(--color-text-muted);
        font-weight: 500;
    }

    &__version-installed {
        font-size: 0.65rem;
        font-weight: 800;
        text-transform: uppercase;
        color: var(--color-green);
        background: color-mix(in srgb, var(--color-green) 15%, transparent);
        padding: 2px 6px;
        border-radius: var(--radius-full);
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

@keyframes spin {
    from {
        transform: rotate(0deg);
    }
    to {
        transform: rotate(360deg);
    }
}

.spin {
    animation: spin 1s linear infinite;
}
</style>
