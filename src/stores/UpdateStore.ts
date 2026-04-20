import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

export type UpdateStatus = {
    message: string;
    type: "success" | "error" | "info" | null;
};

export const useUpdateStore = defineStore("update", () => {
    const availableVersion = ref<string | null>(null);
    const bannerDismissed = ref(false);

    const foundUpdate = ref<Update | null>(null);

    const isChecking = ref(false);
    const isUpdating = ref(false);
    const hasChecked = ref(false);
    const updateStatus = ref<UpdateStatus>({ message: "", type: null });

    const hasUpdate = computed(() => !!availableVersion.value);

    const setAvailableUpdate = (version: string) => {
        availableVersion.value = version;
        bannerDismissed.value = false;
    };

    const dismissBanner = () => {
        bannerDismissed.value = true;
    };

    const clear = () => {
        availableVersion.value = null;
        bannerDismissed.value = false;
        foundUpdate.value = null;
        updateStatus.value = { message: "", type: null };
        hasChecked.value = false;
    };

    const checkForUpdates = async () => {
        if (isChecking.value) return;
        isChecking.value = true;
        updateStatus.value = { message: "Checking for latest client version...", type: "info" };

        try {
            const update = await check();
            if (update) {
                foundUpdate.value = update;
                availableVersion.value = update.version;
                bannerDismissed.value = false;
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
            hasChecked.value = true;
        }
    };

    const installUpdate = async () => {
        if (!foundUpdate.value || isUpdating.value) return;

        isUpdating.value = true;
        updateStatus.value = { message: "Downloading and installing update...", type: "info" };

        try {
            const update = await check();
            await update?.downloadAndInstall();
            updateStatus.value = { message: "Update installed! Relaunching...", type: "success" };
            setTimeout(async () => await relaunch(), 2000);
        } catch (e) {
            console.error("Failed to install update:", e);
            updateStatus.value = { message: "Failed to install update.", type: "error" };
            isUpdating.value = false;
        }
    };

    return {
        availableVersion,
        bannerDismissed,
        foundUpdate,
        isChecking,
        isUpdating,
        hasChecked,
        hasUpdate,
        updateStatus,
        setAvailableUpdate,
        dismissBanner,
        clear,
        checkForUpdates,
        installUpdate,
    };
});
