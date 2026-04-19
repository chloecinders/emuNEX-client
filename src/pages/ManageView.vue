<script setup lang="ts">
import { Download, GamepadDirectional, HardDrive, Plus } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import ConsoleStorageCard from "../components/cards/ConsoleStorageCard.vue";
import EmulatorCard from "../components/cards/EmulatorCard.vue";
import DownloadEmulatorsModal from "../components/modals/DownloadEmulatorsModal.vue";
import EditEmulatorModal from "../components/modals/EditEmulatorModal.vue";
import InstallModal, { type InstallItem } from "../components/modals/InstallModal.vue";
import AlertModal from "../components/ui/AlertModal.vue";
import Heading from "../components/ui/Heading.vue";
import PillButton from "../components/ui/PillButton.vue";
import Spinner from "../components/ui/Spinner.vue";
import Text from "../components/ui/Text.vue";
import { formatBytes } from "../lib/format";
import { useConsoleStore } from "../stores/ConsoleStore";
import { type Emulator, type ServerEmulator, useEmulatorStore } from "../stores/EmulatorStore";
import { useGameStore } from "../stores/GameStore";
import { useRomStore } from "../stores/RomStore";

const router = useRouter();
const consoleStore = useConsoleStore();
const emulatorStore = useEmulatorStore();
const romStore = useRomStore();
const gameStore = useGameStore();

const groupedRoms = computed(() => {
    const groups: Record<string, typeof romStore.installedRoms> = {};
    romStore.installedRoms.forEach((rom) => {
        let console = rom.console;
        if (!console) {
            const game = gameStore.library.find((g) => g.rom_id === rom.game_id);
            console = game?.console || "Unknown";
        }
        if (!groups[console]) groups[console] = [];
        groups[console].push(rom);
    });
    return groups;
});

const sortedConsoles = computed(() => Object.keys(groupedRoms.value).sort());

const getConsoleTotalSize = (consoleName: string) => {
    const total = groupedRoms.value[consoleName]?.reduce((acc, rom) => acc + rom.rom_size + rom.save_size, 0) || 0;
    return formatBytes(total) ?? "0 B";
};

const navigateToConsole = (consoleName: string) => {
    router.push(`/manage/roms/${consoleName}`);
};

const emulatorDirSizes = ref<Record<string, number>>({});

async function refreshDirSizes() {
    try {
        const { invoke } = await import("@tauri-apps/api/core");
        emulatorDirSizes.value = await invoke<Record<string, number>>("get_emulator_dir_sizes");
    } catch (e) {
        console.error("Failed to get emulator dir sizes:", e);
    }
}

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
        await emulatorStore.downloadEmulator(targetConsole, serverEmulator.id);
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

onMounted(async () => {
    await consoleStore.fetchConsoles();
    await Promise.all([romStore.fetchInstalledRoms(), emulatorStore.fetchEmulators(), refreshDirSizes()]);
    if (gameStore.library.length === 0) {
        await gameStore.fetchLibrary();
    }
});
</script>

<template>
    <div class="c-manage">
        <!-- ── Storage section ─────────────────────────────────────────────── -->
        <section class="c-manage__section">
            <div class="c-manage__section-header">
                <Heading :level="2" color="primary" is-badge>
                    <HardDrive :size="16" class="c-manage__section-icon" /> Storage
                </Heading>
            </div>

            <div v-if="romStore.loading" class="c-manage__loading">
                <Spinner />
                <Text>Scanning storage...</Text>
            </div>

            <div v-else-if="sortedConsoles.length === 0" class="c-manage__empty">
                <HardDrive :size="40" class="c-manage__empty-icon" />
                <Text variant="muted">No games installed locally.</Text>
            </div>

            <div v-else class="c-console-grid">
                <ConsoleStorageCard
                    v-for="consoleName in sortedConsoles"
                    :key="consoleName"
                    :console-name="consoleName"
                    :count="groupedRoms[consoleName].length"
                    :total-size="getConsoleTotalSize(consoleName)"
                    :color="consoleStore.getConsoleColor(consoleName) || 'var(--color-primary)'"
                    @click="navigateToConsole(consoleName)"
                />
            </div>
        </section>

        <!-- ── Emulators section ───────────────────────────────────────────── -->
        <section class="c-manage__section">
            <div class="c-manage__section-header">
                <Heading :level="2" color="primary" is-badge>
                    <GamepadDirectional :size="16" class="c-manage__section-icon" /> Emulators
                </Heading>
                <div class="c-manage__section-actions">
                    <PillButton @click="addCustomEmulator()"><Plus /> Add Custom</PillButton>
                    <PillButton @click="openDownloadModal()"><Download /> Download More</PillButton>
                </div>
            </div>

            <div v-if="consoleStore.loading || (emulatorStore.loading && !showDownloadModal)" class="c-manage__loading">
                <Spinner />
                <Text>Loading configurations...</Text>
            </div>

            <div v-else-if="Object.keys(emulatorStore.emulators).length === 0" class="c-manage__empty">
                <GamepadDirectional :size="40" class="c-manage__empty-icon" />
                <Text variant="muted">No emulators configured.</Text>
            </div>

            <div v-else class="c-emulator-grid">
                <EmulatorCard
                    v-for="emulator in Object.values(emulatorStore.emulators)"
                    :key="emulator.id"
                    :emulator="emulator"
                    :dir-size="emulatorDirSizes[emulator.id]"
                    @edit="initEdit(emulator)"
                    @delete="promptDelete(emulator.id)"
                    @set-default="handleSetDefault(emulator.id)"
                />
            </div>
        </section>

        <!-- ── Modals ──────────────────────────────────────────────────────── -->
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
    </div>
</template>

<style lang="scss" scoped>
.c-manage {
    padding: var(--spacing-md) var(--spacing-lg) var(--spacing-xl) var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xxl);

    &__section {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-lg);
    }

    &__section-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    &__section-actions {
        display: flex;
        gap: var(--spacing-sm);
    }

    &__section-icon {
        display: inline;
        vertical-align: middle;
        margin-right: var(--spacing-xs);
    }

    &__loading {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-xl) 0;
    }

    &__empty {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-sm);
        padding: var(--spacing-xl);
        background: var(--color-surface);
        border-radius: var(--radius-lg);
        border: 2px dashed var(--color-border);
    }

    &__empty-icon {
        color: var(--color-text-muted);
        opacity: 0.4;
    }
}

.c-console-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: var(--spacing-lg);
}

.c-emulator-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: var(--spacing-lg);
}
</style>
