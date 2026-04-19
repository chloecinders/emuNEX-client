<script setup lang="ts">
import { Save } from "lucide-vue-next";
import { computed, ref, watch } from "vue";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { type Emulator, useEmulatorStore } from "../../stores/EmulatorStore";
import Button from "../ui/Button.vue";
import Input from "../ui/Input.vue";
import Modal from "../ui/Modal.vue";
import Text from "../ui/Text.vue";

const props = defineProps<{
    show: boolean;
    emulatorId: string | null;
}>();

const emit = defineEmits(["close", "save"]);

const emulatorStore = useEmulatorStore();
const consoleStore = useConsoleStore();

const consoles = computed(() => {
    const serverConsoles = Object.keys(consoleStore.consoles);
    const localConsoles = Object.values(emulatorStore.emulators).flatMap((e) => e.consoles);
    const allConsoles = new Set([...serverConsoles, ...localConsoles]);
    return Array.from(allConsoles).sort();
});

const editState = ref<any>({});
const newSaveExtInputs = ref<Record<string, string>>({});

watch(
    () => props.emulatorId,
    (newId) => {
        if (newId && emulatorStore.emulators[newId]) {
            editState.value[newId] = JSON.parse(
                JSON.stringify({
                    ...emulatorStore.emulators[newId],
                    save_extensions: emulatorStore.emulators[newId].save_extensions ?? [],
                }),
            );
            newSaveExtInputs.value[newId] = "";
        }
    },
    { immediate: true },
);

const addSaveExtToEmulator = (id: string) => {
    const val = (newSaveExtInputs.value[id] || "").trim().replace(/^\./, "");
    if (val && !editState.value[id].save_extensions.includes(val)) {
        editState.value[id].save_extensions.push(val);
    }
    newSaveExtInputs.value[id] = "";
};

const removeSaveExtFromEmulator = (emulatorId: string, ext: string) => {
    editState.value[emulatorId].save_extensions = editState.value[emulatorId].save_extensions.filter(
        (e: string) => e !== ext,
    );
};

const saveEmulatorChanges = async () => {
    if (!props.emulatorId) return;
    const data = editState.value[props.emulatorId];
    if (data && data.name) {
        await emulatorStore.saveEmulator(data as Emulator);
        emit("save", props.emulatorId);
    }
};

const closeModal = () => {
    emit("close");
};
</script>

<template>
    <Modal
        :show="show"
        :title="props.emulatorId && editState[props.emulatorId]?.name ? 'Edit Emulator' : 'Emulator Configuration'"
        width="600px"
        @close="closeModal"
    >
        <div v-if="props.emulatorId && editState[props.emulatorId]" class="c-emulator-form">
            <div class="c-emulator-field">
                <Text variant="label" size="sm">Name</Text>
                <Input v-model="editState[props.emulatorId].name" placeholder="e.g. RetroArch (GBA)" />
            </div>

            <div class="c-emulator-field">
                <Text variant="label" size="sm">Assigned Consoles</Text>
                <div style="display: flex; flex-direction: column; gap: 8px">
                    <div class="c-tag-list">
                        <label
                            v-for="c in consoles"
                            :key="c"
                            class="c-console-checkbox"
                            tabindex="0"
                            role="checkbox"
                            :aria-checked="editState[props.emulatorId].consoles.includes(c)"
                            @keydown.enter.space.prevent="
                                if (editState[props.emulatorId].consoles.includes(c)) {
                                    editState[props.emulatorId].consoles = editState[props.emulatorId].consoles.filter(
                                        (v: string) => v !== c,
                                    );
                                } else {
                                    editState[props.emulatorId].consoles.push(c);
                                }
                            "
                        >
                            <input type="checkbox" :value="c" v-model="editState[props.emulatorId].consoles" />
                            {{ c.toUpperCase() }}
                        </label>
                    </div>
                </div>
            </div>

            <div class="c-emulator-field">
                <Text variant="label" size="sm">Binary Path</Text>
                <Input v-model="editState[props.emulatorId].binary_path" placeholder="Leave blank if in system PATH" />
            </div>

            <div class="c-emulator-field">
                <Text variant="label" size="sm">Run Command</Text>
                <Input v-model="editState[props.emulatorId].run_command" placeholder="e.g. $exe -L core.so $rom" />
            </div>

            <div class="c-emulator-field">
                <Text variant="label" size="sm">Input Config File (optional)</Text>
                <Input
                    v-model="editState[props.emulatorId].input_config_file"
                    placeholder="e.g. %LocalAppData%/vbam/vbam.ini"
                />
            </div>

            <div class="c-emulator-field">
                <Text variant="label" size="sm">Input Mapper (optional)</Text>
                <Input v-model="editState[props.emulatorId].input_mapper" placeholder="e.g. vbam" />
            </div>

            <div class="c-emulator-field">
                <Text variant="label" size="sm">Save Path (optional)</Text>
                <Input v-model="editState[props.emulatorId].save_path" placeholder="e.g. /saves/$rom_name.sav" />
            </div>

            <div class="c-emulator-field">
                <Text variant="label" size="sm">Save File Extensions</Text>
                <div style="display: flex; flex-direction: column; gap: 8px">
                    <div class="c-tag-list">
                        <span
                            v-for="ext in editState[props.emulatorId].save_extensions"
                            :key="ext"
                            class="c-tag c-tag--button"
                            tabindex="0"
                            role="button"
                            :aria-label="`Remove extension ${ext}`"
                            @click="removeSaveExtFromEmulator(props.emulatorId!, ext)"
                            @keydown.enter.space.prevent="removeSaveExtFromEmulator(props.emulatorId!, ext)"
                            >.{{ ext.replace(/^\./, "") }} ×</span
                        >
                        <Text v-if="editState[props.emulatorId].save_extensions.length === 0" variant="muted" size="sm"
                            >No extensions - will use snapshot diffing</Text
                        >
                    </div>
                    <div style="display: flex; gap: 8px; align-items: center">
                        <Input
                            v-model="newSaveExtInputs[props.emulatorId]"
                            placeholder="e.g. sra, srm, eep"
                            style="flex: 1; margin-bottom: 0"
                            @keydown.enter.prevent="addSaveExtToEmulator(props.emulatorId!)"
                        />
                        <Button @click="addSaveExtToEmulator(props.emulatorId!)">Add</Button>
                    </div>
                </div>
            </div>

            <div class="c-emulator-form__actions">
                <Button color="grey" @click="closeModal">Cancel</Button>
                <Button color="primary" @click="saveEmulatorChanges"> <Save :size="16" /> Save </Button>
            </div>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-emulator-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    padding-top: var(--spacing-sm);

    &__actions {
        display: flex;
        justify-content: flex-end;
        gap: var(--spacing-sm);
        margin-top: var(--spacing-sm);
        padding-top: var(--spacing-md);
        border-top: 1px solid var(--color-border);
    }
}

.c-emulator-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
}

.c-tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 4px;
}

.c-tag {
    font-size: 0.72rem;
    font-weight: 800;
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    color: var(--color-text-muted);
    background: var(--color-surface);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    transition: all 0.2s;
    user-select: none;

    &--button {
        cursor: pointer;
        &:hover {
            border-color: var(--color-red);
            color: var(--color-red);
            background: color-mix(in srgb, var(--color-red) 10%, transparent);
        }
        &:focus {
            outline: 2px solid var(--color-red);
            outline-offset: 1px;
        }
    }
}

.c-console-checkbox {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px 12px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    font-weight: 800;
    cursor: pointer;
    color: var(--color-text-muted);
    transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
    user-select: none;

    &:has(input:checked) {
        background: color-mix(in srgb, var(--color-primary) 12%, transparent);
        border-color: var(--color-primary);
        color: var(--color-primary);
        box-shadow: 0 0 0 1px var(--color-primary);
    }

    &:hover:not(:has(input:checked)) {
        border-color: var(--color-text-muted);
        color: var(--color-text);
    }

    &:focus {
        outline: 2px solid var(--color-primary);
        outline-offset: 1px;
        border-color: var(--color-primary);
    }

    input {
        appearance: none;
        display: none;
        margin: 0;
    }
}
</style>
