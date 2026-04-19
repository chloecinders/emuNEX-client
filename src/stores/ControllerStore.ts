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
    emulator_configs: Record<string, unknown>;
};

const AXIS_THRESHOLD = 0.15;
const EMUNEX_ID = "emunex_default";

export const useControllerStore = defineStore("controller", () => {
    const schemes = ref<InputScheme[]>([]);
    const defaultScheme = ref<InputScheme>({
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

    const activeSchemeId = ref<string | null>(null);
    const heldKeys = ref<Set<string>>(new Set());
    const isNavEnabled = ref(true);

    const allSchemes = computed(() => [defaultScheme.value, ...schemes.value]);

    const activeMappings = computed<Record<string, string>>(() => {
        const scheme = allSchemes.value.find((s) => s.id === activeSchemeId.value) || defaultScheme.value;
        return scheme?.mappings ?? {};
    });

    const pressedActions = ref<Record<string, boolean>>({});

    let rafHandle: number | null = null;

    function poll() {
        const gamepads = navigator.getGamepads();
        const next: Record<string, boolean> = {};

        for (const gamepad of gamepads) {
            if (!gamepad) continue;

            const mappings = activeMappings.value;
            const prefix = `Joy${gamepad.index}.`;

            for (let i = 0; i < gamepad.buttons.length; i++) {
                if (gamepad.buttons[i].pressed) {
                    const pid = `${prefix}Btn${i}`;
                    for (const [action, mapping] of Object.entries(mappings)) {
                        if (mapping === pid) next[action] = true;
                    }
                }
            }

            for (let i = 0; i < gamepad.axes.length; i++) {
                if (Math.abs(gamepad.axes[i]) > AXIS_THRESHOLD) {
                    const dir = gamepad.axes[i] > 0 ? "+" : "-";
                    const pid = `${prefix}Axis${i}${dir}`;
                    for (const [action, mapping] of Object.entries(mappings)) {
                        if (mapping === pid) next[action] = true;
                    }
                }
            }
        }

        for (const [action, mapping] of Object.entries(activeMappings.value)) {
            if (mapping.startsWith("Kbd.") && heldKeys.value.has(mapping.slice(4))) {
                next[action] = true;
            }
        }

        pressedActions.value = next;
        rafHandle = requestAnimationFrame(poll);
    }

    const onKeyDown = (e: KeyboardEvent) => heldKeys.value.add(e.code);
    const onKeyUp = (e: KeyboardEvent) => heldKeys.value.delete(e.code);

    async function load() {
        try {
            const data = (await invoke("load_global_inputs")) as GlobalInputConfig;
            schemes.value = data.schemes ?? [];
            activeSchemeId.value = data.active_scheme_id ?? null;
        } catch {
        }
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

    async function saveActiveScheme(id: string | null) {
        activeSchemeId.value = id;
        try {
            await invoke("save_active_input_scheme", { id });
        } catch (e) {
            console.error("Failed to save active scheme preference", e);
        }
    }

    return {
        schemes,
        allSchemes,
        activeSchemeId,
        activeMappings,
        pressedActions,
        isNavEnabled,
        start,
        stop,
        refresh,
        saveActiveScheme,
        EMUNEX_ID,
    };
});
