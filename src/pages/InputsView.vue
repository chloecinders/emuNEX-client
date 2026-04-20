<script setup lang="ts">
/* WARNING: THIS FILE IS A MESS, BLAME CONTROLLER BUTTON DESIGNERS!!! */

import { invoke } from "@tauri-apps/api/core";
import { Check, Gamepad2, Pencil, Play, Plus, Square, Trash2 } from "lucide-vue-next";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import Button from "../components/ui/Button.vue";
import Heading from "../components/ui/Heading.vue";
import IconButton from "../components/ui/IconButton.vue";
import Select from "../components/ui/Select.vue";
import Switch from "../components/ui/Switch.vue";
import Tooltip from "../components/ui/Tooltip.vue";
import { useControllerStore } from "../stores/ControllerStore";

const listeningFor = ref<string | null>(null);
const isSequenceActive = ref(false);
const sequenceIndex = ref(-1);
const isApplying = ref(false);
const isSaving = ref(false);
const consoleFilter = ref("all");

const controllerStore = useControllerStore();

const isButtonVisible = (btn: string) => {
    if (consoleFilter.value === "all") return true;

    const filters: Record<string, string[]> = {
        nes: ["DPad-Up", "DPad-Down", "DPad-Left", "DPad-Right", "X", "A", "Select", "Start"],
        snes: ["DPad-Up", "DPad-Down", "DPad-Left", "DPad-Right", "Y", "X", "B", "A", "L", "R", "Select", "Start"],
        gba: ["DPad-Up", "DPad-Down", "DPad-Left", "DPad-Right", "X", "A", "L", "R", "Select", "Start"],
        n64: [
            "DPad-Up",
            "DPad-Down",
            "DPad-Left",
            "DPad-Right",
            "X",
            "A",
            "L",
            "R",
            "LT",
            "Start",
            "LS Up",
            "LS Down",
            "LS Left",
            "LS Right",
            "RS Up",
            "RS Down",
            "RS Left",
            "RS Right",
        ],
    };

    return filters[consoleFilter.value]?.includes(btn) ?? true;
};

const getOriginalLabel = (btn: string) => {
    if (consoleFilter.value === "all") return null;

    const labels: Record<string, Record<string, string>> = {
        nes: {
            A: "A",
            X: "B",
            Select: "Select",
            Start: "Start",
            "DPad-Up": "Up",
            "DPad-Down": "Down",
            "DPad-Left": "Left",
            "DPad-Right": "Right",
        },
        snes: {
            B: "A",
            A: "B",
            Y: "X",
            X: "Y",
            L: "L",
            R: "R",
            Select: "Select",
            Start: "Start",
            "DPad-Up": "Up",
            "DPad-Down": "Down",
            "DPad-Left": "Left",
            "DPad-Right": "Right",
        },
        gba: {
            A: "A",
            X: "B",
            L: "L",
            R: "R",
            Select: "Select",
            Start: "Start",
            "DPad-Up": "Up",
            "DPad-Down": "Down",
            "DPad-Left": "Left",
            "DPad-Right": "Right",
        },
        n64: {
            A: "A",
            X: "B",
            L: "L",
            R: "R",
            LT: "Z",
            Start: "Start",
            "LS Up": "Analog Up",
            "LS Down": "Analog Down",
            "LS Left": "Analog Left",
            "LS Right": "Analog Right",
            "RS Up": "C-Up",
            "RS Down": "C-Down",
            "RS Left": "C-Left",
            "RS Right": "C-Right",
            "DPad-Up": "Up",
            "DPad-Down": "Down",
            "DPad-Left": "Left",
            "DPad-Right": "Right",
        },
    };

    return labels[consoleFilter.value]?.[btn] || null;
};

const actionLabels = computed(() => {
    switch (layoutMode.value) {
        case "playstation":
            return { up: "△", left: "□", right: "○", down: "×" };
        case "nintendo":
            return { up: "X", left: "Y", right: "A", down: "B" };
        default:
            return { up: "Y", left: "X", right: "B", down: "A" };
    }
});

const shoulderLabels = computed(() => {
    switch (layoutMode.value) {
        case "playstation":
            return { lt: "L2", l: "L1", rt: "R2", r: "R1" };
        case "nintendo":
            return { lt: "ZL", l: "L", rt: "ZR", r: "R" };
        default:
            return { lt: "LT", l: "LB", rt: "RT", r: "RB" };
    }
});

const centerLabels = computed(() => {
    switch (layoutMode.value) {
        case "playstation":
            return { select: "SHARE", start: "OPTIONS" };
        case "nintendo":
            return { select: "-", start: "+" };
        default:
            return { select: "SELECT", start: "START" };
    }
});

const isLSOnTop = computed(() => layoutMode.value !== "playstation");

const lsOrder = computed(() => (consoleFilter.value === "n64" ? 2 : isLSOnTop.value ? 1 : 2));
const dpadOrder = computed(() => (consoleFilter.value === "n64" ? 1 : isLSOnTop.value ? 2 : 1));
const rsOrder = computed(() => (consoleFilter.value === "n64" ? 1 : 2));
const actionOrder = computed(() => (consoleFilter.value === "n64" ? 2 : 1));

type EmulatorInputConfig = {
    scheme_id: string | null;
    auto_apply_on_start: boolean;
};

type SupportedEmulator = {
    id: string;
    name: string;
    input_mapper: string;
};

const activeSchemeId = ref<string | null>(null);
const emulatorConfigs = ref<Record<string, EmulatorInputConfig>>({});
const supportedEmulators = ref<SupportedEmulator[]>([]);

const activeScheme = computed(
    () => controllerStore.allSchemes.find((s) => s.id === activeSchemeId.value) || controllerStore.allSchemes[0],
);

const layoutMode = computed({
    get: () => activeScheme.value?.layout_mode || "xbox",
    set: (v) => {
        if (activeScheme.value) {
            activeScheme.value.layout_mode = v;
        }
    },
});

const mappings = computed(() => {
    if (activeScheme.value) return activeScheme.value.mappings;
    return {
        LT: "Unmapped",
        RT: "Unmapped",
        L: "Unmapped",
        R: "Unmapped",
        "DPad-Up": "Unmapped",
        "DPad-Down": "Unmapped",
        "DPad-Left": "Unmapped",
        "DPad-Right": "Unmapped",
        Y: "Unmapped",
        X: "Unmapped",
        B: "Unmapped",
        A: "Unmapped",
        "LS Up": "Unmapped",
        "LS Down": "Unmapped",
        "LS Left": "Unmapped",
        "LS Right": "Unmapped",
        "LS Click": "Unmapped",
        "RS Up": "Unmapped",
        "RS Down": "Unmapped",
        "RS Left": "Unmapped",
        "RS Right": "Unmapped",
        "RS Click": "Unmapped",
        Select: "Unmapped",
        Start: "Unmapped",
    };
});

const createScheme = () => {
    const id = "scheme-" + Date.now();
    controllerStore.schemes.push({
        id,
        name: `New Scheme ${controllerStore.schemes.length + 1}`,
        layout_mode: "xbox",
        gamepad_id: "",
        mappings: {
            LT: "Unmapped",
            RT: "Unmapped",
            L: "Unmapped",
            R: "Unmapped",
            "DPad-Up": "Unmapped",
            "DPad-Down": "Unmapped",
            "DPad-Left": "Unmapped",
            "DPad-Right": "Unmapped",
            Y: "Unmapped",
            X: "Unmapped",
            B: "Unmapped",
            A: "Unmapped",
            "LS Up": "Unmapped",
            "LS Down": "Unmapped",
            "LS Left": "Unmapped",
            "LS Right": "Unmapped",
            "LS Click": "Unmapped",
            "RS Up": "Unmapped",
            "RS Down": "Unmapped",
            "RS Left": "Unmapped",
            "RS Right": "Unmapped",
            "RS Click": "Unmapped",
            Select: "Unmapped",
            Start: "Unmapped",
        },
    });
    activeSchemeId.value = id;
};

const visibleKeys = computed(() => {
    const keys = [
        "DPad-Up",
        "DPad-Down",
        "DPad-Left",
        "DPad-Right",
        "Y",
        "X",
        "B",
        "A",
        "L",
        "R",
        "LT",
        "RT",
        "Select",
        "Start",
        "Home",
        "LS Click",
        "RS Click",
        "LS Up",
        "LS Down",
        "LS Left",
        "LS Right",
        "RS Up",
        "RS Down",
        "RS Left",
        "RS Right",
    ];
    return keys.filter(isButtonVisible);
});

const startSequenceBind = () => {
    isSequenceActive.value = true;
    sequenceIndex.value = 0;
    listenForInput(visibleKeys.value[0]);
};

const stopSequenceBind = () => {
    isSequenceActive.value = false;
    sequenceIndex.value = -1;
    stopListening(false);
};

const deleteScheme = () => {
    if (!activeSchemeId.value || activeSchemeId.value === controllerStore.EMUNEX_ID) return;
    controllerStore.schemes = controllerStore.schemes.filter((s) => s.id !== activeSchemeId.value);

    for (const emuId of Object.keys(emulatorConfigs.value)) {
        if (emulatorConfigs.value[emuId].scheme_id === activeSchemeId.value) {
            emulatorConfigs.value[emuId].scheme_id = null;
        }
    }

    activeSchemeId.value = controllerStore.EMUNEX_ID;
};

const schemeOptions = computed(() => controllerStore.allSchemes.map((s) => ({ name: s.name, value: s.id })));

const schemeOptionsWithNone = computed(() => [
    { name: "None", value: "" },
    ...controllerStore.allSchemes.map((s) => ({ name: s.name, value: s.id })),
]);

const getEmulatorSchemeId = (emuId: string) => emulatorConfigs.value[emuId]?.scheme_id || "";

const setEmulatorSchemeId = (emuId: string, val: string) => {
    if (emulatorConfigs.value[emuId]) {
        emulatorConfigs.value[emuId].scheme_id = val || null;
    }
};

const isRenamingScheme = ref(false);
const renameSchemeValue = ref("");
const ignoredInitialInputs = ref<Set<string>>(new Set());
const isWaitingForRelease = ref(false);

const startRename = () => {
    if (!activeScheme.value) return;
    renameSchemeValue.value = activeScheme.value.name;
    isRenamingScheme.value = true;
};

const saveRename = () => {
    if (activeScheme.value && renameSchemeValue.value.trim()) {
        activeScheme.value.name = renameSchemeValue.value.trim();
    }
    isRenamingScheme.value = false;
};

let pollFrame: number | null = null;
let bgPollFrame: number | null = null;
let keyboardHandler: ((e: KeyboardEvent) => void) | null = null;
const AXIS_THRESHOLD = 0.5;

const formatMapping = (val: string) => {
    if (!val || val === "Unmapped" || val === "...") return val;

    if (val.startsWith("Kbd.")) {
        const code = val.slice(4);
        if (code.startsWith("Key")) return code.slice(3);
        if (code.startsWith("Digit")) return code.slice(5);
        const labels: Record<string, string> = {
            Space: "SPACE",
            ArrowUp: "↑",
            ArrowDown: "↓",
            ArrowLeft: "←",
            ArrowRight: "→",
            Enter: "ENTER",
            Escape: "ESC",
            Backspace: "BKSP",
            Tab: "TAB",
            ShiftLeft: "L-SHIFT",
            ShiftRight: "R-SHIFT",
            ControlLeft: "L-CTRL",
            ControlRight: "R-CTRL",
            AltLeft: "L-ALT",
            AltRight: "R-ALT",
        };
        return labels[code] ?? code;
    }
    return val;
};

const pressedKeys = ref<Record<string, boolean>>({});
const heldKeyboardKeys = ref<Set<string>>(new Set());

const bgPollGamepad = () => {
    const gamepads = navigator.getGamepads();
    const currentPressed: Record<string, boolean> = {};

    for (const gamepad of gamepads) {
        if (!gamepad) continue;

        for (let i = 0; i < gamepad.buttons.length; i++) {
            if (gamepad.buttons[i].pressed) {
                const physicalId = `Joy${gamepad.index}.Btn${i}`;
                for (const [k, v] of Object.entries(mappings.value)) {
                    if (v === physicalId) currentPressed[k] = true;
                }
            }
        }

        for (let i = 0; i < gamepad.axes.length; i++) {
            if (Math.abs(gamepad.axes[i]) > AXIS_THRESHOLD) {
                const direction = gamepad.axes[i] > 0 ? "+" : "-";
                const physicalId = `Joy${gamepad.index}.Axis${i}${direction}`;
                for (const [k, v] of Object.entries(mappings.value)) {
                    if (v === physicalId) currentPressed[k] = true;
                }
            }
        }
    }

    for (const [k, v] of Object.entries(mappings.value)) {
        if (v.startsWith("Kbd.")) {
            const code = v.slice(4);
            if (heldKeyboardKeys.value.has(code)) {
                currentPressed[k] = true;
            }
        }
    }

    pressedKeys.value = currentPressed;
    bgPollFrame = requestAnimationFrame(bgPollGamepad);
};

const pollGamepad = () => {
    if (!listeningFor.value) return;

    const gamepads = navigator.getGamepads();

    if (isWaitingForRelease.value) {
        let anyPressed = false;
        for (const gamepad of gamepads) {
            if (!gamepad) continue;
            if (gamepad.buttons.some((b) => b.pressed)) anyPressed = true;
            if (gamepad.axes.some((a) => Math.abs(a) > AXIS_THRESHOLD)) anyPressed = true;
            if (anyPressed) break;
        }

        if (heldKeyboardKeys.value.size > 0) anyPressed = true;

        if (!anyPressed) {
            finishStop();
        } else {
            pollFrame = requestAnimationFrame(pollGamepad);
        }
        return;
    }

    let mapped = false;

    for (const gamepad of gamepads) {
        if (!gamepad) continue;

        for (let i = 0; i < gamepad.buttons.length; i++) {
            const id = `Joy${gamepad.index}.Btn${i}`;
            if (gamepad.buttons[i].pressed) {
                if (ignoredInitialInputs.value.has(id)) continue;
                if (activeScheme.value) activeScheme.value.mappings[listeningFor.value] = id;
                mapped = true;
                break;
            } else {
                ignoredInitialInputs.value.delete(id);
            }
        }
        if (mapped) break;

        for (let i = 0; i < gamepad.axes.length; i++) {
            if (Math.abs(gamepad.axes[i]) > AXIS_THRESHOLD) {
                const direction = gamepad.axes[i] > 0 ? "+" : "-";
                const id = `Joy${gamepad.index}.Axis${i}${direction}`;
                if (ignoredInitialInputs.value.has(id)) continue;
                if (activeScheme.value) activeScheme.value.mappings[listeningFor.value] = id;
                mapped = true;
                break;
            }
        }
        if (mapped) break;
    }

    if (mapped) {
        stopListening();
    } else {
        pollFrame = requestAnimationFrame(pollGamepad);
    }
};

const stopListening = (shouldWait = true) => {
    if (pollFrame) {
        cancelAnimationFrame(pollFrame);
        pollFrame = null;
    }
    if (keyboardHandler) {
        document.removeEventListener("keydown", keyboardHandler);
        keyboardHandler = null;
    }

    if (shouldWait) {
        isWaitingForRelease.value = true;
        pollFrame = requestAnimationFrame(pollGamepad);
    } else {
        finishStop();
    }
};

const finishStop = () => {
    if (pollFrame) cancelAnimationFrame(pollFrame);
    pollFrame = null;
    isWaitingForRelease.value = false;
    listeningFor.value = null;
    pressedKeys.value = {};

    if (isSequenceActive.value && sequenceIndex.value + 1 < visibleKeys.value.length) {
        sequenceIndex.value++;
        listenForInput(visibleKeys.value[sequenceIndex.value]);
        return;
    }

    isSequenceActive.value = false;
    sequenceIndex.value = -1;
    controllerStore.isNavEnabled = true;
};

const listenForInput = (key: string) => {
    if (pollFrame) cancelAnimationFrame(pollFrame);
    pollFrame = null;
    if (keyboardHandler) document.removeEventListener("keydown", keyboardHandler);
    keyboardHandler = null;
    isWaitingForRelease.value = false;

    if (!activeScheme.value) return;
    controllerStore.isNavEnabled = false;
    listeningFor.value = key;

    ignoredInitialInputs.value.clear();

    const gamepads = navigator.getGamepads();
    for (const gamepad of gamepads) {
        if (!gamepad) continue;
        for (let i = 0; i < gamepad.buttons.length; i++) {
            if (gamepad.buttons[i].pressed) ignoredInitialInputs.value.add(`Joy${gamepad.index}.Btn${i}`);
        }
        for (let i = 0; i < gamepad.axes.length; i++) {
            if (Math.abs(gamepad.axes[i]) > AXIS_THRESHOLD) {
                const dir = gamepad.axes[i] > 0 ? "+" : "-";
                ignoredInitialInputs.value.add(`Joy${gamepad.index}.Axis${i}${dir}`);
            }
        }
    }

    for (const code of heldKeyboardKeys.value) {
        ignoredInitialInputs.value.add(`Kbd.${code}`);
    }

    activeScheme.value.mappings[key] = "...";

    keyboardHandler = (e: KeyboardEvent) => {
        if (ignoredInitialInputs.value.has(`Kbd.${e.code}`)) return;
        e.preventDefault();

        if (e.code === "Escape") {
            stopListening();
            return;
        }

        if (!listeningFor.value || !activeScheme.value) return;
        activeScheme.value.mappings[listeningFor.value] = `Kbd.${e.code}`;
        stopListening();
    };
    document.addEventListener("keydown", keyboardHandler);

    pollFrame = requestAnimationFrame(pollGamepad);
};

let autoSaveTimeout: any = null;

const autoSaveConfig = async () => {
    if (autoSaveTimeout) clearTimeout(autoSaveTimeout);

    autoSaveTimeout = setTimeout(async () => {
        isSaving.value = true;
        try {
            await invoke("save_global_inputs", {
                config: {
                    schemes: controllerStore.schemes,
                    active_scheme_id: activeSchemeId.value,
                    emulator_configs: emulatorConfigs.value,
                },
            });
        } catch (error) {
            console.error("Auto-save failed:", error);
        } finally {
            isSaving.value = false;
        }
    }, 1000);
};

watch(
    [() => controllerStore.schemes, activeSchemeId, emulatorConfigs],
    () => {
        if (!listeningFor.value) {
            autoSaveConfig();
        }
    },
    { deep: true },
);

const handleGlobalKeyDown = (e: KeyboardEvent) => {
    if (listeningFor.value || isRenamingScheme.value) return;
    heldKeyboardKeys.value.add(e.code);
};

const handleGlobalKeyUp = (e: KeyboardEvent) => {
    heldKeyboardKeys.value.delete(e.code);
    ignoredInitialInputs.value.delete(`Kbd.${e.code}`);
};

onMounted(async () => {
    try {
        const data: any = await invoke("load_global_inputs");
        console.log(data);
        controllerStore.schemes = data.schemes || [];
        activeSchemeId.value = data.active_scheme_id || null;
        emulatorConfigs.value = data.emulator_configs || {};
        supportedEmulators.value = data.supported_emulators || [];
    } catch (e) {}

    window.addEventListener("keydown", handleGlobalKeyDown);
    window.addEventListener("keyup", handleGlobalKeyUp);
    bgPollFrame = requestAnimationFrame(bgPollGamepad);
});

onUnmounted(() => {
    if (bgPollFrame) cancelAnimationFrame(bgPollFrame);
    window.removeEventListener("keydown", handleGlobalKeyDown);
    window.removeEventListener("keyup", handleGlobalKeyUp);

    controllerStore.isNavEnabled = true;
    stopListening();
});

const applySchemeToEmulator = async (emulatorId: string) => {
    const config = emulatorConfigs.value[emulatorId];
    if (!config || !config.scheme_id) return;

    isApplying.value = true;
    try {
        await invoke("apply_scheme_to_emulator", {
            emulatorId: emulatorId,
            schemeId: config.scheme_id,
        });
    } catch (error) {
        console.error("Local config writing failed:", error);
    } finally {
        isApplying.value = false;
    }
};
</script>

<template>
    <div class="c-settings">
        <div class="c-settings__header-wrap">
            <Heading level="2" color="primary" is-badge class="c-settings__title">Global Inputs</Heading>

            <button
                class="c-test-mode-btn"
                :class="{ 'c-test-mode-btn--active': !controllerStore.isNavEnabled }"
                :title="
                    controllerStore.isNavEnabled
                        ? 'Enable Test Mode (disables app navigation so you can test buttons)'
                        : 'Disable Test Mode'
                "
                @click="controllerStore.isNavEnabled = !controllerStore.isNavEnabled"
            >
                <Gamepad2 :size="16" />
                {{ controllerStore.isNavEnabled ? "Test Mode" : "Testing..." }}
            </button>

            <div v-if="isSaving" class="c-save-status">
                <span class="c-save-dot"></span>
                Saving...
            </div>
        </div>

        <div class="c-settings__content">
            <section class="c-settings__section">
                <Heading level="3" color="primary" is-badge class="c-settings__section-title">Mapping</Heading>

                <div class="c-settings__card">
                    <div class="c-settings__card-top">
                        <div class="c-scheme-actions">
                            <template v-if="!isRenamingScheme">
                                <Select
                                    :model-value="activeSchemeId || ''"
                                    :options="schemeOptions"
                                    placeholder="Select scheme..."
                                    @update:model-value="activeSchemeId = $event"
                                />
                                <Tooltip
                                    text="Rename Scheme"
                                    v-if="activeSchemeId && activeSchemeId !== controllerStore.EMUNEX_ID"
                                >
                                    <IconButton class="c-scheme-icon-btn" @click="startRename">
                                        <Pencil :size="18" />
                                    </IconButton>
                                </Tooltip>
                            </template>
                            <template v-else>
                                <input
                                    v-model="renameSchemeValue"
                                    class="c-scheme-rename"
                                    placeholder="Scheme name..."
                                    @keydown.enter="saveRename"
                                    @keydown.escape="isRenamingScheme = false"
                                    autofocus
                                />
                                <Tooltip text="Save Name">
                                    <IconButton class="c-scheme-icon-btn" @click="saveRename">
                                        <Check :size="18" />
                                    </IconButton>
                                </Tooltip>
                            </template>

                            <Tooltip text="New Scheme">
                                <IconButton class="c-scheme-icon-btn" @click="createScheme">
                                    <Plus :size="20" />
                                </IconButton>
                            </Tooltip>

                            <Tooltip
                                text="Delete Scheme"
                                v-if="activeSchemeId && activeSchemeId !== controllerStore.EMUNEX_ID"
                            >
                                <IconButton class="c-scheme-icon-btn" @click="deleteScheme" color="red">
                                    <Trash2 :size="20" />
                                </IconButton>
                            </Tooltip>

                            <Tooltip text="Sequence Bind" v-if="!isSequenceActive && activeSchemeId">
                                <IconButton class="c-scheme-icon-btn" @click="startSequenceBind">
                                    <Play :size="20" />
                                </IconButton>
                            </Tooltip>

                            <Tooltip text="Stop Sequence" v-else-if="isSequenceActive">
                                <IconButton class="c-scheme-icon-btn" @click="stopSequenceBind" color="red">
                                    <Square :size="20" />
                                </IconButton>
                            </Tooltip>
                        </div>
                    </div>

                    <div class="c-settings__layout-tabs" style="margin-bottom: var(--spacing-sm)">
                        <button
                            class="c-layout-tab"
                            :class="{ 'is-active': consoleFilter === 'all' && layoutMode === 'xbox' }"
                            @click="
                                layoutMode = 'xbox';
                                consoleFilter = 'all';
                            "
                            @keydown.enter.space="
                                layoutMode = 'xbox';
                                consoleFilter = 'all';
                            "
                        >
                            Xbox
                        </button>
                        <button
                            class="c-layout-tab"
                            :class="{ 'is-active': consoleFilter === 'all' && layoutMode === 'playstation' }"
                            @click="
                                layoutMode = 'playstation';
                                consoleFilter = 'all';
                            "
                            @keydown.enter.space="
                                layoutMode = 'playstation';
                                consoleFilter = 'all';
                            "
                        >
                            PlayStation
                        </button>
                        <button
                            class="c-layout-tab"
                            :class="{ 'is-active': consoleFilter === 'all' && layoutMode === 'nintendo' }"
                            @click="
                                layoutMode = 'nintendo';
                                consoleFilter = 'all';
                            "
                            @keydown.enter.space="
                                layoutMode = 'nintendo';
                                consoleFilter = 'all';
                            "
                        >
                            Nintendo
                        </button>
                        <div class="c-layout-tab-divider"></div>
                        <button
                            class="c-layout-tab"
                            :class="{ 'is-active': consoleFilter === 'nes' }"
                            @click="consoleFilter = 'nes'"
                            @keydown.enter.space="consoleFilter = 'nes'"
                        >
                            NES/GB
                        </button>
                        <button
                            class="c-layout-tab"
                            :class="{ 'is-active': consoleFilter === 'snes' }"
                            @click="consoleFilter = 'snes'"
                            @keydown.enter.space="consoleFilter = 'snes'"
                        >
                            SNES/NDS
                        </button>
                        <button
                            class="c-layout-tab"
                            :class="{ 'is-active': consoleFilter === 'gba' }"
                            @click="consoleFilter = 'gba'"
                            @keydown.enter.space="consoleFilter = 'gba'"
                        >
                            GBA
                        </button>
                        <button
                            class="c-layout-tab"
                            :class="{ 'is-active': consoleFilter === 'n64' }"
                            @click="consoleFilter = 'n64'"
                            @keydown.enter.space="consoleFilter = 'n64'"
                        >
                            N64
                        </button>
                    </div>

                    <div class="c-gamepad-layout">
                        <div class="c-gp-side c-gp-side-left">
                            <div class="c-gp-shoulders">
                                <div
                                    class="c-gp-btn"
                                    tabindex="0"
                                    role="button"
                                    v-show="isButtonVisible('LT')"
                                    :class="{ 'is-listening': listeningFor === 'LT', 'is-pressed': pressedKeys['LT'] }"
                                    @click="listenForInput('LT')"
                                    >{{ getOriginalLabel("LT") || shoulderLabels.lt }}
                                    <span class="val">{{ formatMapping(mappings["LT"]) }}</span></div
                                >
                                <div
                                    class="c-gp-btn"
                                    tabindex="0"
                                    role="button"
                                    v-show="isButtonVisible('L')"
                                    :class="{ 'is-listening': listeningFor === 'L', 'is-pressed': pressedKeys['L'] }"
                                    @click="listenForInput('L')"
                                    >{{ getOriginalLabel("L") || shoulderLabels.l }}
                                    <span class="val">{{ formatMapping(mappings["L"]) }}</span></div
                                >
                            </div>

                            <div class="c-gp-left-controls">
                                <div class="c-gp-diamond" :style="{ order: lsOrder }" v-show="isButtonVisible('LS Up')">
                                    <div
                                        class="c-gp-btn up"
                                        tabindex="0"
                                        role="button"
                                        :class="{
                                            'is-listening': listeningFor === 'LS Up',
                                            'is-pressed': pressedKeys['LS Up'],
                                        }"
                                        @click="listenForInput('LS Up')"
                                        >{{ getOriginalLabel("LS Up") || "LS-UP" }}
                                        <span class="val">{{ formatMapping(mappings["LS Up"]) }}</span></div
                                    >
                                    <div
                                        class="c-gp-btn left"
                                        tabindex="0"
                                        role="button"
                                        :class="{
                                            'is-listening': listeningFor === 'LS Left',
                                            'is-pressed': pressedKeys['LS Left'],
                                        }"
                                        @click="listenForInput('LS Left')"
                                        >{{ getOriginalLabel("LS Left") || "LS-LEFT" }}
                                        <span class="val">{{ formatMapping(mappings["LS Left"]) }}</span></div
                                    >
                                    <div
                                        class="c-gp-btn right"
                                        tabindex="0"
                                        role="button"
                                        :class="{
                                            'is-listening': listeningFor === 'LS Right',
                                            'is-pressed': pressedKeys['LS Right'],
                                        }"
                                        @click="listenForInput('LS Right')"
                                        >{{ getOriginalLabel("LS Right") || "LS-RIGHT" }}
                                        <span class="val">{{ formatMapping(mappings["LS Right"]) }}</span></div
                                    >
                                    <div
                                        class="c-gp-btn down"
                                        tabindex="0"
                                        role="button"
                                        :class="{
                                            'is-listening': listeningFor === 'LS Down',
                                            'is-pressed': pressedKeys['LS Down'],
                                        }"
                                        @click="listenForInput('LS Down')"
                                        >{{ getOriginalLabel("LS Down") || "LS-DOWN" }}
                                        <span class="val">{{ formatMapping(mappings["LS Down"]) }}</span></div
                                    >
                                    <div
                                        class="c-gp-btn click"
                                        tabindex="0"
                                        role="button"
                                        v-show="isButtonVisible('LS Click')"
                                        :class="{
                                            'is-listening': listeningFor === 'LS Click',
                                            'is-pressed': pressedKeys['LS Click'],
                                        }"
                                        @click="listenForInput('LS Click')"
                                        >{{ getOriginalLabel("LS Click") || "LS-CLK" }}
                                        <span class="val">{{ formatMapping(mappings["LS Click"]) }}</span></div
                                    >
                                </div>

                                <div class="c-gp-diamond" :style="{ order: dpadOrder }">
                                    <div
                                        class="c-gp-btn up"
                                        tabindex="0"
                                        role="button"
                                        v-show="isButtonVisible('DPad-Up')"
                                        :class="{
                                            'is-listening': listeningFor === 'DPad-Up',
                                            'is-pressed': pressedKeys['DPad-Up'],
                                        }"
                                        @click="listenForInput('DPad-Up')"
                                        >{{ getOriginalLabel("DPad-Up") || "D-UP" }}
                                        <span class="val">{{ formatMapping(mappings["DPad-Up"]) }}</span></div
                                    >
                                    <div
                                        class="c-gp-btn left"
                                        tabindex="0"
                                        role="button"
                                        v-show="isButtonVisible('DPad-Left')"
                                        :class="{
                                            'is-listening': listeningFor === 'DPad-Left',
                                            'is-pressed': pressedKeys['DPad-Left'],
                                        }"
                                        @click="listenForInput('DPad-Left')"
                                        >{{ getOriginalLabel("DPad-Left") || "D-LEFT" }}
                                        <span class="val">{{ formatMapping(mappings["DPad-Left"]) }}</span></div
                                    >
                                    <div
                                        class="c-gp-btn right"
                                        tabindex="0"
                                        role="button"
                                        v-show="isButtonVisible('DPad-Right')"
                                        :class="{
                                            'is-listening': listeningFor === 'DPad-Right',
                                            'is-pressed': pressedKeys['DPad-Right'],
                                        }"
                                        @click="listenForInput('DPad-Right')"
                                        >{{ getOriginalLabel("DPad-Right") || "D-RIGHT" }}
                                        <span class="val">{{ formatMapping(mappings["DPad-Right"]) }}</span></div
                                    >
                                    <div
                                        class="c-gp-btn down"
                                        tabindex="0"
                                        role="button"
                                        v-show="isButtonVisible('DPad-Down')"
                                        :class="{
                                            'is-listening': listeningFor === 'DPad-Down',
                                            'is-pressed': pressedKeys['DPad-Down'],
                                        }"
                                        @click="listenForInput('DPad-Down')"
                                        >{{ getOriginalLabel("DPad-Down") || "D-DOWN" }}
                                        <span class="val">{{ formatMapping(mappings["DPad-Down"]) }}</span></div
                                    >
                                </div>
                            </div>
                        </div>

                        <div class="c-gp-center">
                            <div
                                class="c-gp-btn"
                                tabindex="0"
                                role="button"
                                v-show="isButtonVisible('Select')"
                                :class="{
                                    'is-listening': listeningFor === 'Select',
                                    'is-pressed': pressedKeys['Select'],
                                }"
                                @click="listenForInput('Select')"
                                >{{ getOriginalLabel("Select") || centerLabels.select }}
                                <span class="val">{{ formatMapping(mappings["Select"]) }}</span></div
                            >
                            <div
                                class="c-gp-btn"
                                tabindex="0"
                                role="button"
                                v-show="isButtonVisible('Start')"
                                :class="{
                                    'is-listening': listeningFor === 'Start',
                                    'is-pressed': pressedKeys['Start'],
                                }"
                                @click="listenForInput('Start')"
                                >{{ getOriginalLabel("Start") || centerLabels.start }}
                                <span class="val">{{ formatMapping(mappings["Start"]) }}</span></div
                            >
                        </div>

                        <div class="c-gp-side">
                            <div class="c-gp-shoulders">
                                <div
                                    class="c-gp-btn"
                                    tabindex="0"
                                    role="button"
                                    v-show="isButtonVisible('RT')"
                                    :class="{ 'is-listening': listeningFor === 'RT', 'is-pressed': pressedKeys['RT'] }"
                                    @click="listenForInput('RT')"
                                    >{{ getOriginalLabel("RT") || shoulderLabels.rt }}
                                    <span class="val">{{ formatMapping(mappings["RT"]) }}</span></div
                                >
                                <div
                                    class="c-gp-btn"
                                    tabindex="0"
                                    role="button"
                                    v-show="isButtonVisible('R')"
                                    :class="{ 'is-listening': listeningFor === 'R', 'is-pressed': pressedKeys['R'] }"
                                    @click="listenForInput('R')"
                                    >{{ getOriginalLabel("R") || shoulderLabels.r }}
                                    <span class="val">{{ formatMapping(mappings["R"]) }}</span></div
                                >
                            </div>

                            <div class="c-gp-diamond" :style="{ order: actionOrder }">
                                <div
                                    class="c-gp-btn up"
                                    tabindex="0"
                                    role="button"
                                    v-show="isButtonVisible('Y')"
                                    :class="{ 'is-listening': listeningFor === 'Y', 'is-pressed': pressedKeys['Y'] }"
                                    @click="listenForInput('Y')"
                                    >{{ getOriginalLabel("Y") || actionLabels.up }}
                                    <span class="val">{{ formatMapping(mappings["Y"]) }}</span></div
                                >
                                <div
                                    class="c-gp-btn left"
                                    tabindex="0"
                                    role="button"
                                    v-show="isButtonVisible('X')"
                                    :class="{ 'is-listening': listeningFor === 'X', 'is-pressed': pressedKeys['X'] }"
                                    @click="listenForInput('X')"
                                    >{{ getOriginalLabel("X") || actionLabels.left }}
                                    <span class="val">{{ formatMapping(mappings["X"]) }}</span></div
                                >
                                <div
                                    class="c-gp-btn right"
                                    tabindex="0"
                                    role="button"
                                    v-show="isButtonVisible('B')"
                                    :class="{ 'is-listening': listeningFor === 'B', 'is-pressed': pressedKeys['B'] }"
                                    @click="listenForInput('B')"
                                    >{{ getOriginalLabel("B") || actionLabels.right }}
                                    <span class="val">{{ formatMapping(mappings["B"]) }}</span></div
                                >
                                <div
                                    class="c-gp-btn down"
                                    tabindex="0"
                                    role="button"
                                    v-show="isButtonVisible('A')"
                                    :class="{ 'is-listening': listeningFor === 'A', 'is-pressed': pressedKeys['A'] }"
                                    @click="listenForInput('A')"
                                    >{{ getOriginalLabel("A") || actionLabels.down }}
                                    <span class="val">{{ formatMapping(mappings["A"]) }}</span></div
                                >
                            </div>

                            <div class="c-gp-diamond" :style="{ order: rsOrder }" v-show="isButtonVisible('RS Up')">
                                <div
                                    class="c-gp-btn up"
                                    tabindex="0"
                                    role="button"
                                    :class="{
                                        'is-listening': listeningFor === 'RS Up',
                                        'is-pressed': pressedKeys['RS Up'],
                                    }"
                                    @click="listenForInput('RS Up')"
                                    >{{ getOriginalLabel("RS Up") || "RS-UP" }}
                                    <span class="val">{{ formatMapping(mappings["RS Up"]) }}</span></div
                                >
                                <div
                                    class="c-gp-btn left"
                                    tabindex="0"
                                    role="button"
                                    :class="{
                                        'is-listening': listeningFor === 'RS Left',
                                        'is-pressed': pressedKeys['RS Left'],
                                    }"
                                    @click="listenForInput('RS Left')"
                                    >{{ getOriginalLabel("RS Left") || "RS-LEFT" }}
                                    <span class="val">{{ formatMapping(mappings["RS Left"]) }}</span></div
                                >
                                <div
                                    class="c-gp-btn right"
                                    tabindex="0"
                                    role="button"
                                    :class="{
                                        'is-listening': listeningFor === 'RS Right',
                                        'is-pressed': pressedKeys['RS Right'],
                                    }"
                                    @click="listenForInput('RS Right')"
                                    >{{ getOriginalLabel("RS Right") || "RS-RIGHT" }}
                                    <span class="val">{{ formatMapping(mappings["RS Right"]) }}</span></div
                                >
                                <div
                                    class="c-gp-btn down"
                                    tabindex="0"
                                    role="button"
                                    :class="{
                                        'is-listening': listeningFor === 'RS Down',
                                        'is-pressed': pressedKeys['RS Down'],
                                    }"
                                    @click="listenForInput('RS Down')"
                                    >{{ getOriginalLabel("RS Down") || "RS-DOWN" }}
                                    <span class="val">{{ formatMapping(mappings["RS Down"]) }}</span></div
                                >
                                <div
                                    class="c-gp-btn click"
                                    tabindex="0"
                                    role="button"
                                    v-show="isButtonVisible('RS Click')"
                                    :class="{
                                        'is-listening': listeningFor === 'RS Click',
                                        'is-pressed': pressedKeys['RS Click'],
                                    }"
                                    @click="listenForInput('RS Click')"
                                    >{{ getOriginalLabel("RS Click") || "RS-CLK" }}
                                    <span class="val">{{ formatMapping(mappings["RS Click"]) }}</span></div
                                >
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <section class="c-settings__section">
                <Heading level="3" color="primary" is-badge class="c-settings__section-title"
                    >Override Settings</Heading
                >

                <div class="c-settings__card c-emu-override" v-for="emu in supportedEmulators" :key="emu.id">
                    <div class="c-emu-override__row" v-if="emulatorConfigs[emu.id]">
                        <Heading :level="3" class="c-emu-override__name">{{ emu.name }}</Heading>
                        <div class="c-emu-override__scheme-select">
                            <Select
                                :model-value="getEmulatorSchemeId(emu.id)"
                                :options="schemeOptionsWithNone"
                                placeholder="No scheme assigned"
                                @update:model-value="setEmulatorSchemeId(emu.id, $event)"
                            />
                        </div>
                        <Switch v-model="emulatorConfigs[emu.id].auto_apply_on_start" label="Auto Apply" />
                        <Button
                            @click="applySchemeToEmulator(emu.id)"
                            variant="secondary"
                            :disabled="isApplying || !emulatorConfigs[emu.id]?.scheme_id"
                            >Apply Now</Button
                        >
                    </div>
                </div>
            </section>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-settings {
    padding: var(--spacing-md) var(--spacing-lg) var(--spacing-xl) var(--spacing-lg);
    max-width: 1000px;
    margin: 0 auto;
    width: 100%;

    &__header-wrap {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-xxl);
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
        padding: var(--spacing-sm) var(--spacing-md);
        background: var(--color-surface-variant);
        border-left: 3px solid var(--color-primary);

        &--error {
            border-left-color: var(--color-error, #f44336);
        }
        &--success {
            border-left-color: var(--color-success, #4caf50);
        }
    }

    &__layout-tabs {
        display: flex;
        gap: var(--spacing-sm);
        margin-bottom: var(--spacing-xl);
    }
}

.c-scheme-actions {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);

    :deep(.c-select) {
        margin-bottom: 0;
        min-width: 220px;
    }
}

.c-scheme-btns {
    display: flex;
    gap: var(--spacing-xs);
    flex-shrink: 0;
    padding-bottom: 2px;
}

.c-scheme-icon-btn {
    width: 46px;
    height: 46px;
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    flex-shrink: 0;
    align-self: center;
}

.c-scheme-rename {
    flex: 1;
    background: var(--color-surface);
    border: 2px solid var(--color-primary);
    border-radius: var(--radius-md);
    padding: 10px 14px;
    color: var(--color-text);
    font-family: inherit;
    font-size: 0.95rem;
    font-weight: 600;
    outline: none;
    box-shadow: 0 0 0 4px rgba(107, 92, 177, 0.1);

    &::placeholder {
        opacity: 0.4;
    }
}

.c-emu-override {
    &__row {
        display: flex;
        align-items: center;
        gap: var(--spacing-lg);
    }

    &__name {
        flex-shrink: 0;
        min-width: 160px;
    }

    &__scheme-select {
        flex: 1;

        :deep(.c-select) {
            margin-bottom: 0;
        }
    }
}

.c-layout-tab-divider {
    width: 1px;
    background: var(--color-border);
    margin: 4px 8px;
    align-self: stretch;
}

.c-layout-tab {
    padding: 8px 16px;
    border-radius: var(--radius-full);
    font-size: 0.8rem;
    font-weight: 800;
    cursor: pointer;
    background: var(--color-surface-variant);
    color: var(--color-text-muted);
    transition: all 0.2s;
    border: 1px solid var(--color-border);
    user-select: none;

    &:hover {
        border-color: var(--color-primary);
    }

    &.is-active {
        background: rgba(var(--color-primary-rgb), 0.15);
        color: var(--color-primary);
        border-color: var(--color-primary);
    }
}

.c-gamepad-layout {
    display: flex;
    justify-content: center;
    gap: 20px;
    align-items: center;
    background: rgba(0, 0, 0, 0.1);
    border-radius: var(--radius-lg);
    padding: var(--spacing-xxl) var(--spacing-xl);
    position: relative;
    border: none;
    overflow: hidden;
}

.c-gp-side {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    gap: 30px;
    width: 250px;
}

.c-gp-left-controls {
    display: flex;
    flex-direction: column;
    gap: 30px;
}

.c-gp-center {
    position: relative;
    z-index: 1;
    display: flex;
    gap: 15px;
    align-items: flex-end;
    height: 100%;
}

.c-gp-shoulders {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
}

.c-gp-diamond {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    grid-template-rows: 1fr 1fr 1fr;
    gap: 6px;
    justify-items: center;
    align-items: center;

    .up {
        grid-column: 2;
        grid-row: 1;
    }
    .left {
        grid-column: 1;
        grid-row: 2;
    }
    .click {
        grid-column: 2;
        grid-row: 2;
    }
    .right {
        grid-column: 3;
        grid-row: 2;
    }
    .down {
        grid-column: 2;
        grid-row: 3;
    }
}

.c-gp-btn {
    position: relative;
    background: var(--color-surface-variant);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 8px 4px;
    text-align: center;
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 800;
    width: 100%;
    color: var(--color-text);
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    user-select: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    white-space: nowrap;

    &:hover {
        border-color: var(--color-primary);
        transform: translateY(-2px);
    }

    &.is-listening {
        border-color: var(--color-primary);
        background: rgba(var(--color-primary-rgb), 0.15);
        animation: pulse 1s infinite;
    }

    &.is-pressed {
        border-color: var(--color-primary);
        background: rgba(var(--color-primary-rgb), 0.25);
        box-shadow: 0 0 15px rgba(var(--color-primary-rgb), 0.5);
        transform: scale(0.95);
    }
}

.c-gp-btn .val {
    font-family: "Menlo", "Consolas", "Monaco", monospace;
    color: var(--color-text-muted);
    font-size: 0.65rem;
    background: rgba(0, 0, 0, 0.25);
    border-radius: 4px;
    padding: 2px 4px;
    text-overflow: ellipsis;
    overflow: hidden;
    white-space: nowrap;
    transition: all 0.2s;
}

.c-gp-btn.is-pressed .val {
    color: white;
    background: var(--color-primary);
}

@keyframes pulse {
    0% {
        box-shadow: 0 0 0 0 rgba(var(--color-primary-rgb), 0.4);
    }
    70% {
        box-shadow: 0 0 0 6px rgba(var(--color-primary-rgb), 0);
    }
    100% {
        box-shadow: 0 0 0 0 rgba(var(--color-primary-rgb), 0);
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

.c-test-mode-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 14px;
    border-radius: var(--radius-full);
    border: 1.5px solid var(--color-border);
    background: var(--color-surface-variant);
    color: var(--color-text-muted);
    font-family: inherit;
    font-size: 0.8rem;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
    white-space: nowrap;

    &:hover {
        border-color: var(--color-primary);
        color: var(--color-primary);
    }

    &--active {
        border-color: #ef4444;
        color: #ef4444;
        animation: test-pulse 2s ease-in-out infinite;

        &:hover {
            background: color-mix(in srgb, #ef4444 8%, transparent);
            color: #ef4444;
        }
    }
}

@keyframes test-pulse {
    0%,
    100% {
        box-shadow: 0 0 0 0 rgba(239, 68, 68, 0);
    }
    50% {
        box-shadow: 0 0 0 5px rgba(239, 68, 68, 0.15);
    }
}

.c-save-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.72rem;
    font-weight: 800;
    color: var(--color-text-muted);
    background: var(--color-surface-variant);
    padding: 6px 12px;
    border-radius: var(--radius-full);
    border: 1px solid var(--color-border);
}

.c-save-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-primary);
    animation: flash 1.5s infinite;
}

@keyframes flash {
    0%,
    100% {
        opacity: 0.3;
    }
    50% {
        opacity: 1;
    }
}
</style>
