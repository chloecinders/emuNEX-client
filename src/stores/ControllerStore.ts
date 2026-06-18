import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

type InputScheme = {
    id: string;
    name: string;
    mappings: Record<string, string>;
    layout_mode: string;
    gamepad_id: string;
};

type GlobalInputConfig = {
    schemes: InputScheme[];
    active_scheme_id: string | null;
    emulator_configs: Record<string, EmulatorInputConfig>;
};

type EmulatorInputConfig = {
    scheme_id: string | null;
    auto_apply_on_start: boolean;
};

const AXIS_THRESHOLD = 0.15;
export const useControllerStore = defineStore("controller", () => {
    const EMUNEX_ID = "emunex_default";

    const schemes = ref<InputScheme[]>([]);
    const editingSchemeId = ref<string | null>(null);
    const emulatorConfigs = ref<Record<string, EmulatorInputConfig>>({});
    const heldKeys = ref<Set<string>>(new Set());
    const isNavEnabled = ref(true);
    const isLoaded = ref(false);
    let loadPromise: Promise<void> | null = null;

    const allSchemes = computed(() => schemes.value);

    const activeMappings = computed<Record<string, string>>(() => {
        const scheme = schemes.value.find((s) => s.id === editingSchemeId.value) || schemes.value[0];
        return scheme?.mappings ?? {};
    });

    const pressedActions = ref<Record<string, boolean>>({});
    const navActions = ref<Record<string, boolean>>({});

    let rafHandle: number | null = null;

    function poll() {
        const gamepads = navigator.getGamepads();
        const next: Record<string, boolean> = {};
        const nav: Record<string, boolean> = {};

        const activeMaps = activeMappings.value;
        for (const gamepad of gamepads) {
            if (!gamepad) continue;
            const prefix = `Joy${gamepad.index}.`;

            for (let i = 0; i < gamepad.buttons.length; i++) {
                if (gamepad.buttons[i].pressed) {
                    const pid = `${prefix}Btn${i}`;
                    for (const [action, mapping] of Object.entries(activeMaps)) {
                        if (mapping === pid) next[action] = true;
                    }
                }
            }

            for (let i = 0; i < gamepad.axes.length; i++) {
                if (Math.abs(gamepad.axes[i]) > AXIS_THRESHOLD) {
                    const dir = gamepad.axes[i] > 0 ? "+" : "-";
                    const pid = `${prefix}Axis${i}${dir}`;
                    for (const [action, mapping] of Object.entries(activeMaps)) {
                        if (mapping === pid) next[action] = true;
                    }
                }
            }
        }

        for (const [action, mapping] of Object.entries(activeMaps)) {
            if (mapping.startsWith("Kbd.") && heldKeys.value.has(mapping.slice(4))) {
                next[action] = true;
            }
        }

        const navScheme = schemes.value.find(s => s.id === EMUNEX_ID);
        const navMappings = navScheme?.mappings || activeMappings.value;

        for (const gamepad of gamepads) {
            if (!gamepad) continue;
            const prefix = `Joy${gamepad.index}.`;

            for (let i = 0; i < gamepad.buttons.length; i++) {
                if (gamepad.buttons[i].pressed) {
                    const pid = `${prefix}Btn${i}`;
                    for (const [action, mapping] of Object.entries(navMappings)) {
                        if (mapping === pid) nav[action] = true;
                    }
                }
            }

            for (let i = 0; i < gamepad.axes.length; i++) {
                if (Math.abs(gamepad.axes[i]) > AXIS_THRESHOLD) {
                    const dir = gamepad.axes[i] > 0 ? "+" : "-";
                    const pid = `${prefix}Axis${i}${dir}`;
                    for (const [action, mapping] of Object.entries(navMappings)) {
                        if (mapping === pid) nav[action] = true;
                    }
                }
            }
        }

        for (const [action, mapping] of Object.entries(navMappings)) {
            if (mapping.startsWith("Kbd.") && heldKeys.value.has(mapping.slice(4))) {
                nav[action] = true;
            }
        }

        pressedActions.value = next;
        navActions.value = nav;
        rafHandle = requestAnimationFrame(poll);
    }

    const onKeyDown = (e: KeyboardEvent) => heldKeys.value.add(e.code);
    const onKeyUp = (e: KeyboardEvent) => heldKeys.value.delete(e.code);

    async function load() {
        if (isLoaded.value) return;
        if (loadPromise) return loadPromise;

        loadPromise = (async () => {
            try {
                const data = (await invoke("load_global_inputs")) as GlobalInputConfig;
                const loaded = data.schemes ?? [];

                if (loaded.length > 0) {
                    schemes.value = loaded;
                }

                if (!schemes.value.some(s => s.id === EMUNEX_ID)) {
                    schemes.value.unshift({
                        id: EMUNEX_ID,
                        name: "emuNEX (Built-in)",
                        layout_mode: "xbox",
                        gamepad_id: "any",
                        mappings: {
                            "DPad-Up": "Joy0.Btn12",
                            "DPad-Down": "Joy0.Btn13",
                            "DPad-Left": "Joy0.Btn14",
                            "DPad-Right": "Joy0.Btn15",
                            "LS Up": "Joy0.Axis1-",
                            "LS Down": "Joy0.Axis1+",
                            "LS Left": "Joy0.Axis0-",
                            "LS Right": "Joy0.Axis0+",
                            "RS Up": "Joy0.Axis3-",
                            "RS Down": "Joy0.Axis3+",
                            "A": "Joy0.Btn0",
                            "B": "Joy0.Btn1",
                            "X": "Joy0.Btn2",
                            "Y": "Joy0.Btn3",
                            "L": "Joy0.Btn4",
                            "R": "Joy0.Btn5",
                            "ZL": "Joy0.Btn6",
                            "ZR": "Joy0.Btn7",
                            "Select": "Joy0.Btn8",
                            "Start": "Joy0.Btn9",
                            "LS Click": "Joy0.Btn10",
                            "RS Click": "Joy0.Btn11",
                            "confirm": "Kbd.Enter",
                            "back": "Kbd.Backspace",
                            "up": "Kbd.ArrowUp",
                            "down": "Kbd.ArrowDown",
                            "left": "Kbd.ArrowLeft",
                            "right": "Kbd.ArrowRight",
                        },
                    });
                }

                editingSchemeId.value = EMUNEX_ID;
                emulatorConfigs.value = data.emulator_configs ?? {};
                isLoaded.value = true;
                loadPromise = null;
            } catch (e) {
                console.error("Failed to load controller store:", e);
                loadPromise = null;
            }
        })();

        return loadPromise;
    }

    async function start() {
        await load();
        window.addEventListener("keydown", onKeyDown);
        window.addEventListener("keyup", onKeyUp);
        rafHandle = requestAnimationFrame(poll);
    }

    function stop() {
        if (rafHandle !== null) { cancelAnimationFrame(rafHandle); rafHandle = null; }
        window.removeEventListener("keydown", onKeyDown);
        window.removeEventListener("keyup", onKeyUp);
    }

    async function refresh() {
        await load();
    }

    return {
        schemes,
        allSchemes,
        editingSchemeId,
        activeMappings,
        pressedActions,
        navActions,
        isNavEnabled,
        isLoaded,
        start,
        stop,
        refresh,
        EMUNEX_ID,
        load,
        emulatorConfigs
    };
});
