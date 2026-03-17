import { defineStore } from "pinia";
import { ref, watch } from "vue";

export const useThemeStore = defineStore("themeStore", () => {
    const isDark = ref(localStorage.getItem("theme") === "dark");

    watch(isDark, (value) => {
        localStorage.setItem("theme", value ? "dark" : "light");
        applyTheme();
    });

    function applyTheme() {
        document.documentElement.setAttribute("data-theme", isDark.value ? "dark" : "light");
    }

    applyTheme();

    return {
        isDark,
        applyTheme,
    };
});
