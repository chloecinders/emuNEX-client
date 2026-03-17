<script setup lang="ts">
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { Download, RefreshCw } from "lucide-vue-next";
import { onMounted, ref } from "vue";
import Button from "../components/ui/Button.vue";
import Heading from "../components/ui/Heading.vue";
import Switch from "../components/ui/Switch.vue";
import Text from "../components/ui/Text.vue";
import { useThemeStore } from "../stores/ThemeStore";
import { useUserStore } from "../stores/UserStore";

const userStore = useUserStore();
const themeStore = useThemeStore();
const isChecking = ref(false);
const isUpdating = ref(false);
const foundUpdate = ref<Update | null>(null);
const updateStatus = ref<{ message: string; type: "success" | "error" | "info" | null }>({
    message: "",
    type: null,
});

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
        await foundUpdate.value.downloadAndInstall();
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
                        </div>
                    </div>

                    <div class="c-settings__actions">
                        <Button color="grey" size="sm" :disabled="isChecking || isUpdating" @click="checkForUpdates">
                            <RefreshCw :size="16" :class="{ 'c-settings__spin': isChecking }" />
                            <span>Check for Updates</span>
                        </Button>

                        <Button v-if="foundUpdate" color="blue" size="sm" :disabled="isUpdating" @click="installUpdate">
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
