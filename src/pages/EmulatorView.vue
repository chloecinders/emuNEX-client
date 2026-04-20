<script setup lang="ts">
import { Download, Plus } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import EmulatorCard from "../components/cards/EmulatorCard.vue";
import DownloadEmulatorsModal from "../components/modals/DownloadEmulatorsModal.vue";
import EditEmulatorModal from "../components/modals/EditEmulatorModal.vue";
import InstallModal, { type InstallItem } from "../components/modals/InstallModal.vue";
import UpdateEmulatorModal from "../components/modals/UpdateEmulatorModal.vue";
import AlertModal from "../components/ui/AlertModal.vue";
import Heading from "../components/ui/Heading.vue";
import PillButton from "../components/ui/PillButton.vue";
import Spinner from "../components/ui/Spinner.vue";
import Text from "../components/ui/Text.vue";
import { useToast } from "../lib/useToast";
import { useConsoleStore } from "../stores/ConsoleStore";
import { type Emulator, type ServerEmulator, useEmulatorStore } from "../stores/EmulatorStore";

const emulatorStore = useEmulatorStore();
const consoleStore = useConsoleStore();
const toast = useToast();
const emulatorDirSizes = ref<Record<string, number>>({});

async function refreshDirSizes() {
    try {
        const { invoke } = await import("@tauri-apps/api/core");
        emulatorDirSizes.value = await invoke<Record<string, number>>("get_emulator_dir_sizes");
    } catch (e) {
        console.error("Failed to get emulator dir sizes:", e);
    }
}

onMounted(async () => {
    await consoleStore.fetchConsoles();
    await emulatorStore.fetchEmulators();
    await refreshDirSizes();
    emulatorStore.checkForUpdates();
});

const newlyAddedIds = ref<Set<string>>(new Set());
const editingEmulatorId = ref<string | null>(null);

const initEdit = (emulator: Emulator) => {
    editingEmulatorId.value = emulator.id;
};

const handleEditClose = () => {
    if (editingEmulatorId.value && newlyAddedIds.value.has(editingEmulatorId.value)) {
        delete emulatorStore.emulators[editingEmulatorId.value];
        newlyAddedIds.value.delete(editingEmulatorId.value);
    }
    editingEmulatorId.value = null;
};

const handleEditSave = (id: string) => {
    newlyAddedIds.value.delete(id);
    editingEmulatorId.value = null;
};

const handleSetDefault = async (id: string) => {
    await emulatorStore.setDefaultEmulator(id);
};

const syncMode = ref<"update" | "refresh">("update");

const handleRefreshConfig = async (id: string) => {
    const local = emulatorStore.emulators[id];
    if (!local) return;

    try {
        const serverList = await emulatorStore.fetchAllServerEmulators();
        const server = serverList.find((s) => {
            return s.source_server === local.source_server && id.endsWith(`-${s.id}`);
        });

        if (!server) {
            toast.warning("This emulator is not linked to any active server.");
            return;
        }

        pendingUpdateLocal.value = local;
        pendingUpdateServer.value = server;
        syncMode.value = "refresh";
    } catch (e) {
        toast.error("Failed to fetch server config for comparison.");
    }
};

const pendingUpdateLocal = ref<Emulator | null>(null);
const pendingUpdateServer = ref<ServerEmulator | null>(null);

const handleUpdateClick = (emulator: Emulator) => {
    if (emulator.id.startsWith("server-")) {
        pendingUpdateLocal.value = emulator;
        pendingUpdateServer.value = emulatorStore.updatesAvailable[emulator.id];
        syncMode.value = "update";
    }
};

const cancelUpdate = () => {
    pendingUpdateLocal.value = null;
    pendingUpdateServer.value = null;
};

const confirmUpdate = async (keepConfig: boolean) => {
    if (!pendingUpdateLocal.value) return;

    if (syncMode.value === "refresh") {
        if (keepConfig) {
            cancelUpdate();
            return;
        }
        try {
            await emulatorStore.refreshEmulatorConfig(pendingUpdateLocal.value.id);
            toast.success("Configuration refreshed successfully!");
            emulatorStore.checkForUpdates();
        } catch (e) {
            toast.error(`Failed to refresh config: ${e}`);
        } finally {
            cancelUpdate();
        }
        return;
    }

    const parts = pendingUpdateLocal.value.id.split("-");
    const id = parts[parts.length - 1];

    try {
        await emulatorStore.downloadEmulator(
            pendingUpdateLocal.value.consoles[0] || "unknown",
            id,
            keepConfig,
            pendingUpdateLocal.value.source_server,
        );
        toast.success("Emulator updated successfully!");
        emulatorStore.checkForUpdates();
    } catch (e: any) {
        toast.error(`Failed to update emulator: ${e}`);
    } finally {
        cancelUpdate();
    }
};

const addCustomEmulator = () => {
    const id = `custom-${Date.now()}`;
    newlyAddedIds.value.add(id);
    const newEmulator: Emulator = {
        id,
        name: "New Custom Emulator",
        consoles: [],
        is_default: false,
        binary_path: "",
        run_command: "",
        save_path: "",
        save_extensions: [],
        input_config_file: "",
        input_mapper: "",
        zipped: false,
    };

    emulatorStore.emulators[id] = newEmulator;
    editingEmulatorId.value = id;
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

const downloadFromServer = async (serverEmulator: ServerEmulator) => {
    try {
        const targetConsole = (serverEmulator.consoles && serverEmulator.consoles[0]) || "unknown";
        await emulatorStore.downloadEmulator(targetConsole, serverEmulator.id, false, serverEmulator.source_server);
        showDownloadModal.value = false;
        showConfirmDownloadModal.value = false;
        pendingDownloadEmulator.value = null;
    } catch (e) {
        console.error("Failed to download emulator.", e);
    }
};

const installItems = computed<InstallItem[]>(() => {
    if (!pendingDownloadEmulator.value) return [];
    return [
        {
            name: pendingDownloadEmulator.value.name,
            description: `Emulator for ${pendingDownloadEmulator.value.consoles.map((c) => c.toUpperCase()).join(", ")}`,
            size: pendingDownloadEmulator.value.file_size,
            type: "emulator",
        },
    ];
});

const showDeleteModal = ref(false);
const pendingDeleteId = ref("");

const promptDelete = (id: string) => {
    pendingDeleteId.value = id;
    showDeleteModal.value = true;
};

const confirmDelete = async () => {
    await emulatorStore.removeEmulator(pendingDeleteId.value);
    await refreshDirSizes();
    showDeleteModal.value = false;
    pendingDeleteId.value = "";
};
</script>

<template>
    <div class="c-emulator-management">
        <div class="c-emulator-management__header-wrap">
            <Heading :level="2" color="primary" is-badge class="c-emulator-management__badge">
                Manage Emulators
            </Heading>

            <div style="display: flex; gap: 8px">
                <PillButton @click="addCustomEmulator()"> <Plus /> Add Custom </PillButton>
                <PillButton @click="openDownloadModal()"> <Download /> Download More </PillButton>
            </div>
        </div>

        <div
            v-if="(consoleStore.loading || emulatorStore.loading) && Object.keys(emulatorStore.emulators).length === 0"
            class="c-emulator-management__loading"
        >
            <Spinner />
            <Text>Loading configurations...</Text>
        </div>

        <div v-else class="c-emulator-management__content">
            <div class="c-emulator-list">
                <div v-if="Object.keys(emulatorStore.emulators).length === 0" class="c-emulator-empty">
                    <Text variant="muted">No emulators configured.</Text>
                </div>

                <EmulatorCard
                    v-for="emulator in Object.values(emulatorStore.emulators)"
                    :key="emulator.id"
                    :emulator="emulator"
                    :dir-size="emulatorDirSizes[emulator.id]"
                    :has-update="!!emulatorStore.updatesAvailable[emulator.id]"
                    @edit="initEdit(emulator)"
                    @delete="promptDelete(emulator.id)"
                    @set-default="handleSetDefault(emulator.id)"
                    @refresh-config="handleRefreshConfig(emulator.id)"
                    @update="handleUpdateClick(emulator)"
                />
            </div>
        </div>

        <DownloadEmulatorsModal
            :show="showDownloadModal"
            :server-emulators="serverEmulators"
            :is-fetching="isFetchingServer"
            :loading="emulatorStore.loading"
            @close="showDownloadModal = false"
            @download="promptDownload"
        />

        <AlertModal
            :show="showDeleteModal"
            title="Remove Emulator"
            message="Are you sure you want to remove this emulator?"
            confirm-label="Remove"
            confirm-color="red"
            @close="showDeleteModal = false"
            @confirm="confirmDelete"
        />

        <InstallModal
            :show="showConfirmDownloadModal"
            title="Confirm Emulator Installation"
            :items="installItems"
            :loading="emulatorStore.loading"
            @close="cancelConfirmDownload()"
            @confirm="downloadFromServer(pendingDownloadEmulator!)"
        />

        <EditEmulatorModal
            :show="!!editingEmulatorId"
            :emulator-id="editingEmulatorId"
            @close="handleEditClose"
            @save="handleEditSave"
        />

        <UpdateEmulatorModal
            :show="!!pendingUpdateLocal"
            :mode="syncMode"
            :local-emulator="pendingUpdateLocal"
            :server-emulator="pendingUpdateServer"
            :loading="emulatorStore.loading"
            @close="cancelUpdate"
            @confirm="confirmUpdate"
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
</style>
