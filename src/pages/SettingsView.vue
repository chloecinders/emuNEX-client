<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { Activity, Download, FolderOpen, FolderSearch, RefreshCw, Terminal } from "lucide-vue-next";
import { onMounted, onUnmounted, ref } from "vue";
import Button from "../components/ui/Button.vue";
import Heading from "../components/ui/Heading.vue";
import Switch from "../components/ui/Switch.vue";
import Text from "../components/ui/Text.vue";
import { useDevStore } from "../stores/DevStore";
import { useThemeStore } from "../stores/ThemeStore";
import { useUserStore } from "../stores/UserStore";

const userStore = useUserStore();
const themeStore = useThemeStore();
const devStore = useDevStore();
const isChecking = ref(false);

const currentDataDir = ref<string | null>(null);
const isMigrating = ref(false);
const storageStatus = ref<{ message: string; type: "success" | "error" | "info" | null }>({
    message: "",
    type: null,
});

const loadDataDir = async () => {
    try {
        currentDataDir.value = await invoke<string>("get_data_dir");
    } catch (e) {
        console.error("Failed to get data dir:", e);
    }
};

const handleChangeLocation = async () => {
    const selected = await invoke<string | null>("pick_directory");
    if (!selected || selected === currentDataDir.value) return;

    isMigrating.value = true;
    storageStatus.value = { message: "Moving files to new location\u2026", type: "info" };

    try {
        await invoke("set_custom_data_path", { newPath: selected });
        storageStatus.value = { message: "Done! Reloading\u2026", type: "success" };
        setTimeout(() => window.location.reload(), 1200);
    } catch (e) {
        storageStatus.value = { message: String(e), type: "error" };
        isMigrating.value = false;
    }
};

const handleOpenDataDir = async () => {
    try {
        await invoke("open_data_dir");
    } catch (e) {
        console.error("Failed to open data dir:", e);
    }
};
const isUpdating = ref(false);
const foundUpdate = ref<Update | null>(null);
const updateStatus = ref<{ message: string; type: "success" | "error" | "info" | null }>({
    message: "",
    type: null,
});
const currentVersion = ref<string>("");
const konamiToast = ref<string | null>(null);

const KONAMI = [
    "ArrowUp",
    "ArrowUp",
    "ArrowDown",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
    "ArrowLeft",
    "ArrowRight",
    "b",
    "a",
];
const konamiProgress = ref<string[]>([]);

const handleKeydown = (e: KeyboardEvent) => {
    const expected = KONAMI[konamiProgress.value.length];
    if (e.key === expected) {
        konamiProgress.value.push(e.key);
        if (konamiProgress.value.length === KONAMI.length) {
            konamiProgress.value = [];
            devStore.isDevMode = !devStore.isDevMode;
            konamiToast.value = devStore.isDevMode ? "Developer Mode Enabled" : "Developer Mode Disabled";
            setTimeout(() => (konamiToast.value = null), 3000);
        }
    } else {
        konamiProgress.value = e.key === KONAMI[0] ? [e.key] : [];
    }
};

const openRequestViewer = () => {
    import("@tauri-apps/api/webviewWindow").then(({ WebviewWindow }) => {
        const window = new WebviewWindow("requests", {
            url: "index.html?dev=requests",
            title: "Request Viewer",
            width: 900,
            height: 650,
            decorations: false,
        });

        window.once("tauri://created", () => {
            window.show();
        });

        window.once("tauri://error", (e) => {
            console.error(e);
        });
    });
};

const checkForUpdates = async () => {
    isChecking.value = true;
    updateStatus.value = { message: "Checking for latest client version...", type: "info" };

    try {
        const update = await check();
        if (update) {
            foundUpdate.value = update;
            updateStatus.value = {
                message: `New version ${update.version} available!`,
                type: "success",
            };
        } else {
            foundUpdate.value = null;
            updateStatus.value = { message: "You are running the latest version.", type: "success" };
        }
    } catch (e) {
        console.error("Failed to check for updates:", e);
        updateStatus.value = { message: "Failed to check for updates. Please try again later.", type: "error" };
    } finally {
        isChecking.value = false;
    }
};

const installUpdate = async () => {
    if (!foundUpdate.value) return;

    isUpdating.value = true;
    updateStatus.value = { message: "Downloading and installing update...", type: "info" };

    try {
        // we have to fetch the update again due to a typescript error
        const update = await check();
        await update?.downloadAndInstall();
        updateStatus.value = { message: "Update installed! Relaunching...", type: "success" };
        setTimeout(async () => {
            await relaunch();
        }, 2000);
    } catch (e) {
        console.error("Failed to install update:", e);
        updateStatus.value = { message: "Failed to install update.", type: "error" };
        isUpdating.value = false;
    }
};

onMounted(async () => {
    if (!userStore.user) {
        await userStore.fetchUser();
    }

    try {
        currentVersion.value = await getVersion();
    } catch (e) {
        console.error("Failed to get client version:", e);
    }

    await loadDataDir();

    window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
    window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
    <div class="c-settings">
        <div class="c-settings__header-wrap">
            <Heading level="2" color="primary" is-badge class="c-settings__title">Settings</Heading>
        </div>

        <div class="c-settings__content">
            <section class="c-settings__section">
                <Heading level="3" color="primary" is-badge class="c-settings__section-title">Application</Heading>

                <div class="c-settings__card">
                    <div class="c-settings__card-top">
                        <div class="c-settings__description-wrap">
                            <Heading :level="3">Client Updates</Heading>
                            <Text v-if="currentVersion" variant="muted" size="sm"
                                >Current Version: {{ currentVersion }}</Text
                            >
                        </div>
                    </div>

                    <div class="c-settings__actions">
                        <Button color="grey" size="sm" :disabled="isChecking || isUpdating" @click="checkForUpdates">
                            <RefreshCw :size="16" :class="{ 'c-settings__spin': isChecking }" />
                            <span>Check for Updates</span>
                        </Button>

                        <Button
                            v-if="foundUpdate"
                            color="primary"
                            size="sm"
                            :disabled="isUpdating"
                            @click="installUpdate"
                        >
                            <Download :size="16" />
                            <span>Install Updates</span>
                        </Button>
                    </div>

                    <Transition name="fade">
                        <div
                            v-if="updateStatus.message"
                            class="c-settings__status"
                            :class="`c-settings__status--${updateStatus.type}`"
                        >
                            <Text size="sm">{{ updateStatus.message }}</Text>
                        </div>
                    </Transition>
                </div>
            </section>

            <section class="c-settings__section">
                <Heading level="3" color="primary" is-badge class="c-settings__section-title">Appearance</Heading>

                <div class="c-settings__card">
                    <Switch v-model="themeStore.isDark" label="Dark Mode" />
                </div>
            </section>

            <section class="c-settings__section">
                <Heading level="3" color="primary" is-badge class="c-settings__section-title">Storage</Heading>

                <div class="c-settings__card">
                    <div class="c-settings__card-top">
                        <div class="c-settings__description-wrap">
                            <Heading :level="3">Data Location</Heading>
                            <Text variant="muted" size="sm">ROMs, saves, and emulators are stored here.</Text>
                        </div>
                    </div>

                    <div class="c-settings__path-display">
                        <FolderOpen :size="14" class="c-settings__path-icon" />
                        <code class="c-settings__path-text">{{ currentDataDir ?? "…" }}</code>
                    </div>

                    <div class="c-settings__actions" style="margin-top: var(--spacing-lg)">
                        <Button color="grey" size="sm" :disabled="isMigrating" @click="handleChangeLocation">
                            <FolderSearch :size="16" />
                            <span>{{ isMigrating ? "Moving files…" : "Change Location" }}</span>
                        </Button>
                        <Button color="grey" size="sm" :disabled="isMigrating" @click="handleOpenDataDir">
                            <FolderOpen :size="16" />
                            <span>Open Folder</span>
                        </Button>
                    </div>

                    <Transition name="fade">
                        <div
                            v-if="storageStatus.message"
                            class="c-settings__status"
                            :class="`c-settings__status--${storageStatus.type}`"
                        >
                            <Text size="sm">{{ storageStatus.message }}</Text>
                        </div>
                    </Transition>
                </div>
            </section>

            <Transition name="fade">
                <section v-if="devStore.isDevMode" class="c-settings__section">
                    <Heading level="3" color="primary" is-badge class="c-settings__section-title">
                        <Terminal :size="14" style="display: inline; vertical-align: middle; margin-right: 4px" />
                        Developer
                    </Heading>

                    <div class="c-settings__card c-settings__card--dev">
                        <div class="c-settings__card-top">
                            <div class="c-settings__description-wrap">
                                <Heading :level="3">Request Viewer</Heading>
                                <Text variant="muted" size="sm"
                                    >Inspect all API requests made through the Rust HTTP bridge. Opens in a new
                                    window.</Text
                                >
                            </div>
                        </div>
                        <div class="c-settings__actions">
                            <Button color="grey" size="sm" @click="openRequestViewer">
                                <Activity :size="16" />
                                <span>Open Request Viewer</span>
                            </Button>
                        </div>
                        <div class="c-settings__dev-badge">{{ devStore.requests.length }} requests logged</div>
                    </div>
                </section>
            </Transition>
        </div>

        <Transition name="konami-toast">
            <div v-if="konamiToast" class="c-settings__konami-toast">
                {{ konamiToast }}
            </div>
        </Transition>
    </div>
</template>

<style lang="scss" scoped>
.c-settings {
    padding: var(--spacing-md) var(--spacing-lg) var(--spacing-xl) var(--spacing-lg);
    max-width: 900px;
    margin: 0 auto;
    width: 100%;

    &__header-wrap {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-xxl);
    }

    &__icon {
        color: var(--color-primary);
    }

    &__content {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xxl);
    }

    &__section {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-md);
    }
    &__card {
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-lg);
        padding: var(--spacing-lg);
        box-shadow: var(--shadow-sm);
    }

    &__path-display {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        background: var(--color-surface-variant);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        padding: var(--spacing-sm) var(--spacing-md);
        overflow: hidden;
    }

    &__path-icon {
        color: var(--color-text-muted);
        flex-shrink: 0;
    }

    &__path-text {
        font-family: "Menlo", "Consolas", "Monaco", monospace;
        font-size: 0.78rem;
        color: var(--color-text-muted);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    &__info-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: var(--spacing-md) 0;
        border-bottom: 1px solid var(--color-border);

        &:last-child {
            border-bottom: none;
        }
    }

    &__role-badge {
        padding: 4px 12px;
        border-radius: var(--radius-full);
        font-size: 0.7rem;
        font-weight: 900;
        text-transform: uppercase;
        background: var(--color-surface-variant);
        color: var(--color-text-muted);
        border: 1px solid var(--color-border);

        &--admin {
            border-color: var(--color-primary);
            color: var(--color-primary);
            background: rgba(var(--color-primary-rgb), 0.05);
        }
    }

    &__card-top {
        margin-bottom: var(--spacing-lg);
    }

    &__description-wrap {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    &__actions {
        display: flex;
        gap: var(--spacing-md);
    }

    &__status {
        margin-top: var(--spacing-lg);
        border-radius: var(--radius-md);
    }

    &__spin {
        animation: spin 1.5s linear infinite;
    }

    &__card--dev {
        border-color: rgba(251, 146, 60, 0.4);
        box-shadow:
            0 0 0 1px rgba(251, 146, 60, 0.1),
            var(--shadow-sm);
    }

    &__dev-badge {
        margin-top: var(--spacing-md);
        font-size: 0.7rem;
        color: var(--color-text-muted);
        font-variant-numeric: tabular-nums;
    }

    &__konami-toast {
        position: fixed;
        bottom: 24px;
        right: 24px;
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-lg);
        padding: 12px 18px;
        font-size: 0.85rem;
        font-weight: 600;
        box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
        z-index: 9999;
        pointer-events: none;
    }
}

@keyframes spin {
    from {
        transform: rotate(0deg);
    }
    to {
        transform: rotate(360deg);
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

.konami-toast-enter-active {
    transition:
        opacity 0.3s ease,
        transform 0.3s ease;
}
.konami-toast-leave-active {
    transition:
        opacity 0.4s ease,
        transform 0.4s ease;
}
.konami-toast-enter-from,
.konami-toast-leave-to {
    opacity: 0;
    transform: translateY(12px);
}
</style>
