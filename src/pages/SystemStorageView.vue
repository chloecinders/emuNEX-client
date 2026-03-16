<script setup lang="ts">
import { ArrowLeft, HardDrive, Trash2 } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import Badge from "../components/ui/Badge.vue";
import Button from "../components/ui/Button.vue";
import Heading from "../components/ui/Heading.vue";
import Modal from "../components/ui/Modal.vue";
import PillButton from "../components/ui/PillButton.vue";
import Spinner from "../components/ui/Spinner.vue";
import Text from "../components/ui/Text.vue";
import { useConsoleStore } from "../stores/ConsoleStore";
import { useGameStore } from "../stores/GameStore";
import { useRomStore, type LocalStorageEntry } from "../stores/RomStore";

const route = useRoute();
const router = useRouter();
const romStore = useRomStore();
const consoleStore = useConsoleStore();
const gameStore = useGameStore();

const consoleName = computed(() => route.params.console as string);

onMounted(async () => {
    await consoleStore.fetchConsoles();
    await romStore.fetchInstalledRoms();
    if (gameStore.library.length === 0) {
        await gameStore.fetchLibrary();
    }
});

const formatBytes = (bytes: number, decimals = 2) => {
    if (!+bytes) return "0 Bytes";
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
};

const getGameTitle = (gameId: string) => {
    const game = gameStore.library.find((g) => g.rom_id === gameId);
    return game ? game.title : `Unknown Game (${gameId})`;
};

const systemRoms = computed(() => {
    return romStore.installedRoms.filter((rom) => {
        if (rom.console) {
            return rom.console === consoleName.value;
        }
        const game = gameStore.library.find((g) => g.rom_id === rom.game_id);
        return game?.console === consoleName.value;
    });
});

const showDeleteRomModal = ref(false);
const showDeleteSaveModal = ref(false);
const selectedRom = ref<LocalStorageEntry | null>(null);

const confirmDeleteRom = (rom: LocalStorageEntry) => {
    selectedRom.value = rom;
    showDeleteRomModal.value = true;
};

const confirmDeleteSave = (rom: LocalStorageEntry) => {
    selectedRom.value = rom;
    showDeleteSaveModal.value = true;
};

const executeDeleteRom = async () => {
    if (selectedRom.value) {
        await romStore.deleteRom(selectedRom.value.game_id, consoleName.value);
        showDeleteRomModal.value = false;
        selectedRom.value = null;
        if (systemRoms.value.length === 0) {
            router.push("/manage/roms");
        }
    }
};

const executeDeleteSave = async () => {
    if (selectedRom.value) {
        await romStore.deleteSave(selectedRom.value.game_id);
        showDeleteSaveModal.value = false;
        selectedRom.value = null;
    }
};
</script>

<template>
    <div class="c-system-detail">
        <div class="c-system-detail__header-wrap">
            <div class="c-system-detail__badge">
                <Heading :level="2" class="c-system-detail__title">{{ consoleName.toUpperCase() }} Storage</Heading>
                <span class="c-system-detail__count">{{ systemRoms.length }} Installed</span>
            </div>

            <PillButton @click="router.push('/manage/roms')"> <ArrowLeft /> Back </PillButton>
        </div>

        <div v-if="romStore.loading" class="c-system-detail__loading">
            <Spinner />
            <Text>Scanning metadata...</Text>
        </div>

        <div v-else-if="systemRoms.length === 0" class="c-system-detail__empty">
            <HardDrive :size="48" class="c-system-detail__empty-icon" />
            <Heading :level="3">No Local ROMs</Heading>
            <Text variant="muted">You don't have any games installed for {{ consoleName.toUpperCase() }}.</Text>
        </div>

        <div v-else class="c-rom-list">
            <div v-for="rom in systemRoms" :key="rom.game_id" class="c-rom-card">
                <div class="c-rom-card__info">
                    <Heading :level="3" class="c-rom-card__title">{{ getGameTitle(rom.game_id) }}</Heading>
                    <div class="c-rom-card__meta">
                        <Badge v-if="rom.rom_size > 0" color="grey">ROM: {{ formatBytes(rom.rom_size) }}</Badge>
                        <Badge v-else color="red">ROM Missing</Badge>
                        <Badge v-if="rom.save_size > 0" color="blue"
                            >Local Save: {{ formatBytes(rom.save_size) }}</Badge
                        >
                    </div>
                </div>

                <div class="c-rom-card__actions">
                    <Button v-if="rom.save_size > 0" color="grey" size="sm" @click="confirmDeleteSave(rom)">
                        <Trash2 :size="16" /> Delete Save
                    </Button>
                    <Button v-if="rom.rom_size > 0" color="red" size="sm" @click="confirmDeleteRom(rom)">
                        <Trash2 :size="16" /> Delete ROM
                    </Button>
                </div>
            </div>
        </div>

        <Modal :show="showDeleteRomModal" title="Delete ROM File" @close="showDeleteRomModal = false">
            <div class="c-delete-modal">
                <Text
                    >Are you sure you want to permanently delete the game file for
                    <strong>{{ selectedRom ? getGameTitle(selectedRom.game_id) : "" }}</strong
                    >?</Text
                >

                <div class="c-delete-modal__actions">
                    <Button color="grey" @click="showDeleteRomModal = false">Cancel</Button>
                    <Button color="red" @click="executeDeleteRom">Delete ROM</Button>
                </div>
            </div>
        </Modal>

        <Modal :show="showDeleteSaveModal" title="Delete Local Save Data" @close="showDeleteSaveModal = false">
            <div class="c-delete-modal">
                <Text
                    >Are you sure you want to delete the local save data for
                    <strong>{{ selectedRom ? getGameTitle(selectedRom.game_id) : "" }}</strong
                    >?</Text
                >
                <Text variant="muted" size="sm"
                    >This will permanently erase your progress unless it was already synchronized to the cloud.</Text
                >

                <div class="c-delete-modal__actions">
                    <Button color="grey" @click="showDeleteSaveModal = false">Cancel</Button>
                    <Button color="red" @click="executeDeleteSave">Delete</Button>
                </div>
            </div>
        </Modal>
    </div>
</template>

<style lang="scss" scoped>
.c-system-detail {
    padding: var(--spacing-md) var(--spacing-lg) var(--spacing-xl) var(--spacing-lg);

    &__header-wrap {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-xl);
    }

    &__badge {
        display: inline-flex;
        align-items: center;
        gap: var(--spacing-md);
        background: var(--color-surface-variant);
        padding: var(--spacing-sm) var(--spacing-md);
        border-radius: var(--radius-full);
        border: 1px solid var(--color-border);
    }

    &__title {
        color: var(--color-primary);
        font-size: 1.1rem;
        font-weight: 800;
        margin: 0;
    }

    &__count {
        font-size: 0.75rem;
        font-weight: 800;
        color: var(--color-text-muted);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    &__loading,
    &__empty {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-xxl) 0;
    }

    &__empty-icon {
        color: var(--color-text-muted);
        opacity: 0.5;
        margin-bottom: var(--spacing-md);
    }
}

.c-rom-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(450px, 1fr));
    gap: var(--spacing-lg);
}

.c-rom-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--spacing-lg);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    box-shadow: var(--shadow-sm);

    &__info {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }

    &__title {
        margin: 0;
        font-weight: 800;
        color: var(--color-text);
        line-height: 1.2;
    }

    &__meta {
        display: flex;
        gap: var(--spacing-sm);
        flex-wrap: wrap;
    }

    &__actions {
        display: flex;
        gap: var(--spacing-sm);
        justify-content: flex-end;
        align-items: center;
        padding-top: var(--spacing-md);
        margin-top: auto;
    }
}

.c-delete-modal {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--spacing-md);
    padding: var(--spacing-md) 0;

    &__actions {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-md);
        width: 100%;
        margin-top: var(--spacing-lg);
    }
}
</style>
