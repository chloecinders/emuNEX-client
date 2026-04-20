import { defineStore } from "pinia";
import { ref, watch } from "vue";

export interface AccentPreset {
    id: string;
    label: string;
    hue: number;
    chroma: number;
}

export const ACCENT_PRESETS: AccentPreset[] = [
    { id: "violet", label: "Violet", hue: 289, chroma: 0.13 },
    { id: "blue", label: "Blue", hue: 250, chroma: 0.15 },
    { id: "cyan", label: "Cyan", hue: 200, chroma: 0.14 },
    { id: "emerald", label: "Emerald", hue: 155, chroma: 0.16 },
    { id: "amber", label: "Amber", hue: 70, chroma: 0.18 },
    { id: "rose", label: "Rose", hue: 15, chroma: 0.19 },
];

export const useThemeStore = defineStore("themeStore", () => {
    const isDark = ref(localStorage.getItem("theme") === "dark");
    const accentId = ref(localStorage.getItem("accent") ?? "violet");
    const customHue = ref(Number(localStorage.getItem("theme_custom_hue") ?? "200"));

    function currentPreset(): AccentPreset | null {
        return ACCENT_PRESETS.find((p) => p.id === accentId.value) ?? null;
    }

    function applyAccent() {
        const p = currentPreset();
        let h: number;
        let c: number;

        if (p) {
            h = p.hue;
            c = p.chroma;
        } else {
            h = customHue.value;
            c = 0.15;
        }

        const root = document.documentElement.style;

        root.setProperty("--accent-hue", h.toString());
        root.setProperty("--accent-chroma", c.toString());
        root.setProperty("--base-hue", h.toString());
    }

    function applyTheme() {
        document.documentElement.setAttribute("data-theme", isDark.value ? "dark" : "light");
        applyAccent();
    }

    function setAccent(id: string) {
        accentId.value = id;
        localStorage.setItem("accent", id);
        applyAccent();
    }

    function setCustomHue(hue: number) {
        customHue.value = hue;
        localStorage.setItem("theme_custom_hue", hue.toString());
        if (accentId.value === "custom") {
            applyAccent();
        }
    }

    watch(isDark, (value) => {
        localStorage.setItem("theme", value ? "dark" : "light");
        applyTheme();
    });

    applyTheme();

    return {
        isDark,
        accentId,
        customHue,
        accentPresets: ACCENT_PRESETS,
        currentPreset,
        setAccent,
        setCustomHue,
        applyTheme,
    };
});
