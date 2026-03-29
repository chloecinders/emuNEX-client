<script setup lang="ts">
import { ArrowLeft, HardDrive } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import RomCard from "../components/cards/RomCard.vue";
import AlertModal from "../components/ui/AlertModal.vue";
import Heading from "../components/ui/Heading.vue";
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

    systemRoms.value.forEach(async (rom) => {
        const status = await romStore.checkSaveStatus(rom.game_id);
        if (status?.latest_version !== null && status?.latest_version !== undefined) {
            cloudSaves.value[rom.game_id] = status.latest_version;
        }
    });
});

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

const cloudSaves = ref<Record<string, number>>({});

const showDeleteRomModal = ref(false);
const selectedRomForDelete = ref<LocalStorageEntry | null>(null);

const promptDeleteRom = (rom: LocalStorageEntry) => {
    selectedRomForDelete.value = rom;
    showDeleteRomModal.value = true;
};

const executeDeleteRom = async () => {
    if (selectedRomForDelete.value) {
        await romStore.deleteRom(selectedRomForDelete.value.game_id, consoleName.value);
        showDeleteRomModal.value = false;
        selectedRomForDelete.value = null;
        if (systemRoms.value.length === 0) {
            router.push("/manage/roms");
        }
    }
};

const showDeleteSaveModal = ref(false);
const selectedRomForSave = ref<LocalStorageEntry | null>(null);

const promptDeleteSave = (rom: LocalStorageEntry) => {
    selectedRomForSave.value = rom;
    showDeleteSaveModal.value = true;
};

const executeDeleteSave = async () => {
    if (selectedRomForSave.value) {
        await romStore.deleteSave(selectedRomForSave.value.game_id);
        showDeleteSaveModal.value = false;
        selectedRomForSave.value = null;
    }
};
</script>

<template>
    <div class="c-system-detail">
        <div class="c-system-detail__header-wrap">
            <Heading :level="2" color="primary" is-badge class="c-system-detail__badge">
                {{ consoleName.toUpperCase() }} Storage
                <span class="c-system-detail__count">{{ systemRoms.length }} Installed</span>
            </Heading>

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
            <RomCard
                v-for="rom in systemRoms"
                :key="rom.game_id"
                :rom="rom"
                :game-title="getGameTitle(rom.game_id)"
                :cloud-version="cloudSaves[rom.game_id]"
                @delete-rom="promptDeleteRom(rom)"
                @delete-save="promptDeleteSave(rom)"
            />
        </div>

        <AlertModal
            :show="showDeleteRomModal"
            title="Delete ROM File"
            :message="`Are you sure you want to permanently delete the game file for <strong>${selectedRomForDelete ? getGameTitle(selectedRomForDelete.game_id) : ''}</strong>?`"
            confirm-label="Delete ROM"
            confirm-color="red"
            @close="showDeleteRomModal = false"
            @confirm="executeDeleteRom"
        />

        <AlertModal
            :show="showDeleteSaveModal"
            title="Delete Local Save Data"
            :message="`Are you sure you want to delete the local save data for <strong>${selectedRomForSave ? getGameTitle(selectedRomForSave.game_id) : ''}</strong>?<br><br><small>This will permanently erase your progress unless it was already synchronized to the cloud.</small>`"
            confirm-label="Delete"
            confirm-color="red"
            @close="showDeleteSaveModal = false"
            @confirm="executeDeleteSave"
        />
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
        margin-top: calc(var(--spacing-sm) * -1);
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
</style>
