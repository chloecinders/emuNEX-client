<script setup lang="ts">
import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { Activity, Download, FolderOpen, FolderSearch, Pipette, RefreshCw, Terminal } from "lucide-vue-next";
import { onMounted, onUnmounted, ref } from "vue";
import Button from "../components/ui/Button.vue";
import Heading from "../components/ui/Heading.vue";
import Switch from "../components/ui/Switch.vue";
import Text from "../components/ui/Text.vue";
import { useToast } from "../lib/useToast";
import { useDevStore } from "../stores/DevStore";
import { useThemeStore } from "../stores/ThemeStore";
import { useUpdateStore } from "../stores/UpdateStore";
import { useUserStore } from "../stores/UserStore";

const userStore = useUserStore();
const themeStore = useThemeStore();
const devStore = useDevStore();
const updateStore = useUpdateStore();

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
const toast = useToast();
const currentVersion = ref<string>("");

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
            toast.info(devStore.isDevMode ? "Developer Mode Enabled" : "Developer Mode Disabled");
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

    if (!updateStore.hasChecked) {
        updateStore.checkForUpdates();
    }

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
                        <Button
                            color="grey"
                            size="sm"
                            :disabled="updateStore.isChecking || updateStore.isUpdating"
                            @click="updateStore.checkForUpdates()"
                        >
                            <RefreshCw :size="16" :class="{ 'c-settings__spin': updateStore.isChecking }" />
                            <span>Check for Updates</span>
                        </Button>

                        <Button
                            v-if="updateStore.foundUpdate"
                            color="primary"
                            size="sm"
                            :disabled="updateStore.isUpdating"
                            @click="updateStore.installUpdate()"
                        >
                            <Download :size="16" />
                            <span>Install Updates</span>
                        </Button>
                    </div>

                    <Transition name="fade">
                        <div
                            v-if="updateStore.updateStatus.message"
                            class="c-settings__status"
                            :class="`c-settings__status--${updateStore.updateStatus.type}`"
                        >
                            <Text size="sm">{{ updateStore.updateStatus.message }}</Text>
                        </div>
                    </Transition>
                </div>
            </section>

            <section class="c-settings__section">
                <Heading level="3" color="primary" is-badge class="c-settings__section-title">Appearance</Heading>

                <div class="c-settings__card">
                    <Switch v-model="themeStore.isDark" label="Dark Mode" />

                    <div class="c-settings__divider" />

                    <div class="c-settings__description-wrap" style="margin-bottom: var(--spacing-md)">
                        <Heading :level="3">Accent Color</Heading>
                    </div>

                    <div class="c-accent-swatches">
                        <button
                            v-for="preset in themeStore.accentPresets"
                            :key="preset.id"
                            class="c-accent-swatch"
                            :class="{ 'c-accent-swatch--active': themeStore.accentId === preset.id }"
                            :style="{ background: `oklch(0.59 ${preset.chroma} ${preset.hue})` }"
                            :title="preset.label"
                            :aria-label="preset.label"
                            :aria-pressed="themeStore.accentId === preset.id"
                            @click="themeStore.setAccent(preset.id)"
                        />

                        <button
                            class="c-accent-swatch c-accent-swatch--custom"
                            :class="{ 'c-accent-swatch--active': themeStore.accentId === 'custom' }"
                            :style="{ background: `oklch(0.59 0.15 ${themeStore.customHue})` }"
                            title="Custom Color"
                            aria-label="Custom Color"
                            :aria-pressed="themeStore.accentId === 'custom'"
                            @click="themeStore.setAccent('custom')"
                        >
                            <Pipette :size="14" />
                        </button>
                    </div>

                    <div v-if="themeStore.accentId === 'custom'" class="c-custom-accent">
                        <div class="c-custom-accent__header">
                            <Text size="sm" variant="muted">Hue: {{ themeStore.customHue }}°</Text>
                        </div>
                        <input
                            type="range"
                            min="0"
                            max="360"
                            step="1"
                            :value="themeStore.customHue"
                            class="c-hue-slider"
                            @input="(e) => themeStore.setCustomHue(Number((e.target as HTMLInputElement).value))"
                        />
                    </div>
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

    &__divider {
        height: 1px;
        background: var(--color-border);
        margin: var(--spacing-lg) 0;
        opacity: 0.5;
    }
}

.c-accent-swatches {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-md);
}

.c-accent-swatch {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-full);
    border: 3px solid transparent;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    padding: 0;

    &:hover {
        transform: scale(1.15);
    }

    &--active {
        border-color: white;
        box-shadow:
            0 0 0 2px var(--color-primary),
            0 0 15px rgba(var(--color-primary-rgb), 0.4);
        transform: scale(1.1);
    }

    &--custom {
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
    }

    body.is-controller-navigating &:focus {
        outline-offset: 4px;
        transform: scale(1.2);
    }
}

.c-custom-accent {
    margin-top: var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);

    &__header {
        display: flex;
        justify-content: flex-end;
    }
}

.c-hue-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 8px;
    border-radius: var(--radius-full);
    background: linear-gradient(
        to right,
        oklch(0.6 0.15 0),
        oklch(0.6 0.15 60),
        oklch(0.6 0.15 120),
        oklch(0.6 0.15 180),
        oklch(0.6 0.15 240),
        oklch(0.6 0.15 300),
        oklch(0.6 0.15 360)
    );
    outline: none;

    &::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 18px;
        height: 18px;
        border-radius: 50%;
        background: white;
        border: 2px solid var(--color-primary);
        cursor: pointer;
        box-shadow: var(--shadow-sm);
        transition: transform 0.15s ease;

        &:hover {
            transform: scale(1.2);
        }
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
</style>
