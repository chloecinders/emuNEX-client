<script setup lang="ts">
import { HardDrive } from "lucide-vue-next";
import { computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import ConsoleStorageCard from "../components/cards/ConsoleStorageCard.vue";
import Heading from "../components/ui/Heading.vue";
import Spinner from "../components/ui/Spinner.vue";
import Text from "../components/ui/Text.vue";
import { useConsoleStore } from "../stores/ConsoleStore";
import { useGameStore } from "../stores/GameStore";
import { useRomStore } from "../stores/RomStore";
import { formatBytes } from "../lib/format";

const router = useRouter();
const romStore = useRomStore();
const consoleStore = useConsoleStore();
const gameStore = useGameStore();

onMounted(async () => {
    await consoleStore.fetchConsoles();
    await romStore.fetchInstalledRoms();
    if (gameStore.library.length === 0) {
        await gameStore.fetchLibrary();
    }
});

const groupedRoms = computed(() => {
    const groups: Record<string, typeof romStore.installedRoms> = {};
    romStore.installedRoms.forEach((rom) => {
        let console = rom.console;
        if (!console) {
            const game = gameStore.library.find((g) => g.rom_id === rom.game_id);
            console = game?.console || "Unknown";
        }
        if (!groups[console]) {
            groups[console] = [];
        }
        groups[console].push(rom);
    });
    return groups;
});

const sortedConsoles = computed(() => {
    return Object.keys(groupedRoms.value).sort();
});

const getConsoleTotalSize = (consoleName: string) => {
    const total =
        groupedRoms.value[consoleName]?.reduce((acc, rom) => acc + rom.rom_size + rom.save_size, 0) || 0;
    return formatBytes(total) ?? "0 B";
};

const navigateToConsole = (consoleName: string) => {
    router.push(`/manage/roms/${consoleName}`);
};
</script>

<template>
    <div class="c-rom-management">
        <div class="c-rom-management__header-wrap">
            <Heading :level="2" color="primary" is-badge class="c-rom-management__badge">
                Storage
            </Heading>
        </div>

        <div v-if="romStore.loading" class="c-rom-management__loading">
            <Spinner />
            <Text>Scanning storage...</Text>
        </div>

        <div v-else-if="sortedConsoles.length === 0" class="c-rom-management__empty">
            <HardDrive :size="48" class="c-rom-management__empty-icon" />
            <Heading :level="3">Storage Empty</Heading>
            <Text variant="muted">You don't have any games installed locally.</Text>
        </div>

        <div v-else class="c-rom-management__content">
            <div class="c-console-grid">
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
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-rom-management {
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

.c-console-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: var(--spacing-lg);
}
</style>
