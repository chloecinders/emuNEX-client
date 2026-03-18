import { computed, ref } from "vue";
import { defineStore } from "pinia";

export const useUpdateStore = defineStore("update", () => {
    const availableVersion = ref<string | null>(null);
    const bannerDismissed = ref(false);

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
    };

    return {
        availableVersion,
        bannerDismissed,
        hasUpdate,
        setAvailableUpdate,
        dismissBanner,
        clear,
    };
});


