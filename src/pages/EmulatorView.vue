<script setup lang="ts">
import { CheckCircle, Download, Pencil, Plus, Save, Trash2 } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import InstallModal, { type InstallItem } from "../components/modals/InstallModal.vue";
import Badge from "../components/ui/Badge.vue";
import Button from "../components/ui/Button.vue";
import Heading from "../components/ui/Heading.vue";
import IconButton from "../components/ui/IconButton.vue";
import Input from "../components/ui/Input.vue";
import Modal from "../components/ui/Modal.vue";
import PillButton from "../components/ui/PillButton.vue";
import Spinner from "../components/ui/Spinner.vue";
import Text from "../components/ui/Text.vue";
import Tooltip from "../components/ui/Tooltip.vue";
import { useConsoleStore } from "../stores/ConsoleStore";
import { type Emulator, type ServerEmulator, useEmulatorStore } from "../stores/EmulatorStore";

const emulatorStore = useEmulatorStore();
const consoleStore = useConsoleStore();

onMounted(async () => {
    await consoleStore.fetchConsoles();
    await emulatorStore.fetchEmulators();
});

const consoles = computed(() => {
    const serverConsoles = Object.keys(consoleStore.consoles).map((c) => c.toLowerCase());
    const localConsoles = Object.values(emulatorStore.emulators).flatMap((e) => e.consoles);
    const allConsoles = new Set([...serverConsoles, ...localConsoles]);
    return Array.from(allConsoles).sort();
});

const editState = ref<any>({});
const newlyAddedIds = ref<Set<string>>(new Set());

const newConsoleInputs = ref<Record<string, string>>({});

const addConsoleToEmulator = (id: string) => {
    const val = (newConsoleInputs.value[id] || "").trim().toLowerCase();
    if (val && !editState.value[id].consoles.includes(val)) {
        editState.value[id].consoles.push(val);
    }
    newConsoleInputs.value[id] = "";
};

const initEdit = (emulator: Emulator) => {
    editState.value[emulator.id] = JSON.parse(JSON.stringify(emulator));
    newConsoleInputs.value[emulator.id] = "";
};

const saveEmulatorChanges = async (id: string) => {
    const data = editState.value[id];
    if (data && data.name) {
        await emulatorStore.saveEmulator(data as Emulator);
        newlyAddedIds.value.delete(id);
        delete editState.value[id];
    }
};

const cancelEdit = (id: string) => {
    delete editState.value[id];

    if (newlyAddedIds.value.has(id)) {
        delete emulatorStore.emulators[id];
        newlyAddedIds.value.delete(id);
    }
};

const handleSetDefault = async (id: string) => {
    await emulatorStore.setDefaultEmulator(id);
};

const addCustomEmulator = () => {
    const id = `custom-${Date.now()}`;
    newlyAddedIds.value.add(id);
    const newEmulator: Emulator = {
        id,
        name: "New Custom Emulator",
        consoles: [],
        is_default: false,
        is_installed: true,
        binary_path: "",
        run_command: "",
        save_path: "",
        config_files: [],
        zipped: false,
    };

    emulatorStore.emulators[id] = newEmulator;
    editState.value[id] = { ...newEmulator };
};

const showDownloadModal = ref(false);
const serverEmulators = ref<ServerEmulator[]>([]);
const isFetchingServer = ref(false);

const openDownloadModal = async () => {
    showDownloadModal.value = true;
    isFetchingServer.value = true;
    serverEmulators.value = await emulatorStore.fetchAllServerEmulators();
    isFetchingServer.value = false;
};

const downloadFromServer = async (serverEmulator: ServerEmulator) => {
    try {
        const targetConsole = (serverEmulator.consoles && serverEmulator.consoles[0]) || "unknown";
        await emulatorStore.downloadEmulator(targetConsole, serverEmulator.id);
        showDownloadModal.value = false;
        showConfirmDownloadModal.value = false;
        pendingDownloadEmulator.value = null;
    } catch (e) {
        console.error("Failed to download emulator.", e);
    }
};

const showConfirmDownloadModal = ref(false);
const pendingDownloadEmulator = ref<ServerEmulator | null>(null);

const promptDownload = (serverEmulator: ServerEmulator) => {
    pendingDownloadEmulator.value = serverEmulator;
    showDownloadModal.value = false;
    showConfirmDownloadModal.value = true;
};

const cancelConfirmDownload = () => {
    showConfirmDownloadModal.value = false;
    showDownloadModal.value = true;
    pendingDownloadEmulator.value = null;
};

const showDeleteModal = ref(false);
const pendingDeleteId = ref("");

const promptDelete = (id: string) => {
    pendingDeleteId.value = id;
    showDeleteModal.value = true;
};

const confirmDelete = async () => {
    await emulatorStore.removeEmulator(pendingDeleteId.value);
    showDeleteModal.value = false;
    pendingDeleteId.value = "";
};

const installItems = computed<InstallItem[]>(() => {
    if (!pendingDownloadEmulator.value) return [];
    return [
        {
            name: pendingDownloadEmulator.value.name,
            description: `Emulator for ${pendingDownloadEmulator.value.consoles.map(c=>c.toUpperCase()).join(', ')}`,
            size: pendingDownloadEmulator.value.file_size,
            type: "emulator",
        },
    ];
});
</script>

<template>
    <div class="c-emulator-management">
        <div class="c-emulator-management__header-wrap">
            <Heading :level="2" color="primary" is-badge class="c-emulator-management__badge">
                Manage Emulators
            </Heading>
            
            <div style="display: flex; gap: 8px;">
                <PillButton @click="addCustomEmulator()"> <Plus /> Add Custom </PillButton>
                <PillButton @click="openDownloadModal()"> <Download /> Download More </PillButton>
            </div>
        </div>

        <div
            v-if="consoleStore.loading || (emulatorStore.loading && !showDownloadModal)"
            class="c-emulator-management__loading"
        >
            <Spinner />
            <Text>Loading configurations...</Text>
        </div>

        <div v-else class="c-emulator-management__content">
            <div class="c-emulator-list">
                <div
                    v-if="Object.keys(emulatorStore.emulators).length === 0"
                    class="c-emulator-empty"
                >
                    <Text variant="muted">No emulators configured.</Text>
                </div>

                <div
                    v-for="emulator in Object.values(emulatorStore.emulators)"
                    :key="emulator.id"
                    class="c-emulator-card"
                    :class="{
                        'c-emulator-card--default': emulator.is_default,
                        'c-emulator-card--editing': editState[emulator.id],
                    }"
                >
                    <div class="c-emulator-card__header">
                            <div class="c-emulator-card__title-area">
                                <Input
                                    v-if="editState[emulator.id]"
                                    v-model="editState[emulator.id].name"
                                    placeholder="e.g. RetroArch (GBA)"
                                />
                                <Heading v-else :level="3" class="c-emulator-card__name">{{
                                    emulator.name || "Unnamed Emulator"
                                }}</Heading>
                                <Badge v-if="emulator.is_default" color="green">Default</Badge>
                            </div>

                            <div class="c-emulator-card__actions">
                                <template v-if="editState[emulator.id]">
                                    <Button color="grey" size="sm" @click="cancelEdit(emulator.id)"
                                        >Cancel</Button
                                    >
                                    <Button
                                        color="primary"
                                        size="sm"
                                        @click="saveEmulatorChanges(emulator.id)"
                                    >
                                        <Save :size="16" /> Save
                                    </Button>
                                </template>

                                <Tooltip v-if="!editState[emulator.id]" text="Make Default">
                                    <IconButton
                                        v-if="!emulator.is_default"
                                        @click="handleSetDefault(emulator.id)"
                                    >
                                        <CheckCircle />
                                    </IconButton>
                                </Tooltip>

                                <Tooltip v-if="!editState[emulator.id]" text="Edit">
                                    <IconButton @click="initEdit(emulator)">
                                        <Pencil />
                                    </IconButton>
                                </Tooltip>

                                <Tooltip v-if="!editState[emulator.id]" text="Delete">
                                    <IconButton color="red" @click="promptDelete(emulator.id)">
                                        <Trash2 />
                                    </IconButton>
                                </Tooltip>
                            </div>
                        </div>

                        <div class="c-emulator-card__body">
                            <div class="c-emulator-field">
                                <Text variant="label" size="sm">Assigned Consoles</Text>
                                <div v-if="editState[emulator.id]" style="display: flex; flex-direction: column; gap: 8px;">
                                    <div style="display: flex; gap: 12px; flex-wrap: wrap; margin-top: 4px;">
                                        <label v-for="c in consoles" :key="c" style="display: flex; align-items: center; gap: 4px; font-size: 0.85rem;">
                                            <input 
                                                type="checkbox" 
                                                :value="c" 
                                                v-model="editState[emulator.id].consoles"
                                            />
                                            {{ c.toUpperCase() }}
                                        </label>
                                    </div>
                                    <div style="display: flex; gap: 8px; align-items: center;">
                                        <Input 
                                            v-model="newConsoleInputs[emulator.id]" 
                                            placeholder="Add custom console..." 
                                            style="flex: 1;"
                                        />
                                        <Button size="sm" @click="addConsoleToEmulator(emulator.id)">Add</Button>
                                    </div>
                                </div>
                                <div v-else style="display: flex; gap: 8px; flex-wrap: wrap; margin-top: 4px;">
                                    <Badge v-for="c in emulator.consoles" :key="c" color="grey" size="sm">{{ c.toUpperCase() }}</Badge>
                                </div>
                            </div>

                            <div class="c-emulator-field">
                                <Text variant="label" size="sm">Binary Path</Text>
                                <Input
                                    v-if="editState[emulator.id]"
                                    v-model="editState[emulator.id].binary_path"
                                    placeholder="Leave blank if in system PATH"
                                />
                                <Text v-else variant="muted" class="c-emulator-field__value">{{
                                    emulator.binary_path || "No binary path set"
                                }}</Text>
                            </div>

                            <div class="c-emulator-field">
                                <Text variant="label" size="sm">Run Command</Text>
                                <Input
                                    v-if="editState[emulator.id]"
                                    v-model="editState[emulator.id].run_command"
                                    placeholder="e.g. $exe -L core.so $rom"
                                />
                                <Text v-else variant="muted" class="c-emulator-field__value">{{
                                    emulator.run_command || "Not set"
                                }}</Text>
                            </div>

                            <div class="c-emulator-field">
                                <Text variant="label" size="sm">Save Path (optional)</Text>
                                <Input
                                    v-if="editState[emulator.id]"
                                    v-model="editState[emulator.id].save_path"
                                    placeholder="e.g. /saves/$rom_name.sav"
                                />
                                <Text v-else variant="muted" class="c-emulator-field__value">{{
                                    emulator.save_path || "No custom save path"
                                }}</Text>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        <Modal :show="showDownloadModal" title="Download Emulators" @close="showDownloadModal = false">
            <div class="c-download-modal">
                <div v-if="isFetchingServer" class="c-download-modal__loading">
                    <Spinner />
                    <Text>Fetching available emulators...</Text>
                </div>
                <div v-else-if="serverEmulators.length === 0" class="c-download-modal__empty">
                    <Text variant="muted">No emulators available for download on the server.</Text>
                </div>
                <div v-else class="c-download-list">
                    <div v-for="se in serverEmulators" :key="se.id" class="c-download-item">
                        <div class="c-download-item__info">
                            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 4px; flex-wrap: wrap;">
                                <Heading :level="4" class="c-download-item__title">{{ se.name }}</Heading>
                                <Badge v-for="c in se.consoles" :key="c" color="blue" size="sm">{{ c.toUpperCase() }}</Badge>
                            </div>
                            <Text variant="muted" size="sm">Platform: {{ se.platform }}</Text>
                        </div>
                        <Button @click="promptDownload(se)" :disabled="emulatorStore.loading" color="primary">
                            <template v-if="emulatorStore.loading">Downloading...</template>
                            <template v-else>Download</template>
                        </Button>
                    </div>
                </div>
            </div>
        </Modal>

        <Modal :show="showDeleteModal" title="Remove Emulator" @close="showDeleteModal = false">
            <div class="c-delete-modal">
                <Text>Are you sure you want to remove this emulator?</Text>
                <div class="c-delete-modal__actions">
                    <Button color="grey" @click="showDeleteModal = false">Cancel</Button>
                    <Button color="red" @click="confirmDelete">Remove</Button>
                </div>
            </div>
        </Modal>

        <InstallModal
            :show="showConfirmDownloadModal"
            title="Confirm Emulator Installation"
            :items="installItems"
            :loading="emulatorStore.loading"
            @close="cancelConfirmDownload()"
            @confirm="downloadFromServer(pendingDownloadEmulator!)"
        />
    </div>
</template>

<style lang="scss" scoped>
.c-emulator-management {
    padding: var(--spacing-md) var(--spacing-lg) var(--spacing-xl) var(--spacing-lg);

    &__header-wrap {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-xl);
    }

    &__badge {
        margin-top: calc(var(--spacing-sm) * -1);
    }

    &__loading {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-xxl) 0;
    }
}

.c-console-section {
    margin-bottom: var(--spacing-xxl);

    &__header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-lg);
    }

    &__title {
        display: inline-flex;
        align-items: center;
    }

    &__count {
        font-size: 0.75rem;
        font-weight: 800;
        color: var(--color-text-muted);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    &__actions {
        display: flex;
        gap: var(--spacing-sm);
    }
}

.c-emulator-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: var(--spacing-lg);
}

.c-emulator-empty {
    text-align: center;
    padding: var(--spacing-xl);
    background: var(--color-surface);
    border-radius: var(--radius-lg);
    border: 2px dashed var(--color-border);
    color: var(--color-text-muted);
    font-weight: 600;
}

.c-emulator-card {
    background: var(--color-surface-variant);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    transition: all 0.25s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    box-shadow: var(--shadow-sm);
    position: relative;

    &--editing {
        .c-emulator-card__header {
            flex-direction: column-reverse;
            align-items: stretch;
            gap: var(--spacing-sm);
        }

        .c-emulator-card__actions {
            justify-content: flex-end;
            padding-bottom: var(--spacing-sm);
            width: 100%;
        }
    }

    &:hover {
        border-color: var(--color-primary);
        box-shadow: var(--shadow-md);
        transform: translateY(-2px);
    }

    &--default {
        background: var(--color-surface-variant);
    }

    &__header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: var(--spacing-md);
        margin-bottom: var(--spacing-xs);
    }

    &__title-area {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: var(--spacing-xs);
        flex: 1;
    }

    &__name {
        margin: 0;
        font-weight: 800;
        color: var(--color-text);
        line-height: 1.2;
    }

    &__actions {
        display: flex;
        gap: var(--spacing-xs);
        flex-wrap: wrap;
        justify-content: flex-end;
    }

    &__body {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-md);
        flex-grow: 1;
    }
}

.c-emulator-field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);

    &__value {
        font-family: inherit;
        font-size: 0.85rem;
        word-break: break-all;
        background: var(--color-surface-variant);
        padding: var(--spacing-sm);
        border-radius: var(--radius-sm);
        border: 1px solid var(--color-border);
        color: var(--color-text-muted);
    }
}

.c-download-modal {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);

    &__loading,
    &__empty {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-xl) 0;
    }
}

.c-download-list {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
}

.c-download-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: all 0.2s ease;

    &:hover {
        border-color: var(--color-primary);
        background: var(--color-surface-hover);
    }

    &__info {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs);
    }

    &__title {
        margin: 0;
        font-weight: 700;
    }
}

.c-delete-modal {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) 0;

    &__actions {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-md);
        width: 100%;
        margin-top: var(--spacing-sm);

        :deep(.c-button) {
            width: 100%;
        }
    }
}
</style>
