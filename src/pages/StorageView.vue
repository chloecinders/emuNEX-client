<script setup lang="ts">
import { ChevronRight, HardDrive } from "lucide-vue-next";
import { computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import Heading from "../components/ui/Heading.vue";
import Spinner from "../components/ui/Spinner.vue";
import Text from "../components/ui/Text.vue";
import { useConsoleStore } from "../stores/ConsoleStore";
import { useGameStore } from "../stores/GameStore";
import { useRomStore } from "../stores/RomStore";

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

const formatBytes = (bytes: number, decimals = 2) => {
    if (!+bytes) return "0 Bytes";
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`;
};

const getConsoleTotalSize = (consoleName: string) => {
    const total = groupedRoms.value[consoleName]?.reduce((acc, rom) => acc + rom.rom_size + rom.save_size, 0) || 0;
    return formatBytes(total);
};

const navigateToConsole = (consoleName: string) => {
    router.push(`/manage/roms/${consoleName}`);
};
</script>

<template>
    <div class="c-rom-management">
        <div class="c-rom-management__header-wrap">
            <div class="c-rom-management__badge">
                <Heading :level="1" class="c-rom-management__title">Storage</Heading>
            </div>
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
                <div
                    v-for="consoleName in sortedConsoles"
                    :key="consoleName"
                    class="c-console-card"
                    @click="navigateToConsole(consoleName)"
                >
                    <div
                        class="c-console-card__bg-indicator"
                        :style="{
                            backgroundColor: consoleStore.getConsoleColor(consoleName) || 'var(--color-primary)',
                        }"
                    ></div>

                    <div class="c-console-card__content">
                        <div class="c-console-card__info">
                            <div class="c-console-card__badge-wrapper">
                                <div class="c-console-card__badge">
                                    <Heading :level="2" class="c-console-card__title">
                                        {{ consoleName.toUpperCase() }}
                                    </Heading>
                                    <span class="c-console-card__count"
                                        >{{ groupedRoms[consoleName].length }} Installed</span
                                    >
                                </div>
                            </div>
                            <Text variant="muted" size="sm"
                                >Total space used: {{ getConsoleTotalSize(consoleName) }}</Text
                            >
                        </div>
                        <ChevronRight :size="24" class="c-console-card__arrow" />
                    </div>
                </div>
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

.c-console-card {
    position: relative;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    cursor: pointer;
    transition: all 0.25s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    box-shadow: var(--shadow-sm);
    display: flex;
    flex-direction: column;

    &:hover {
        border-color: var(--color-primary-light);
        box-shadow: var(--shadow-md);
        transform: translateY(-2px);

        .c-console-card__arrow {
            transform: translateX(4px);
            color: var(--color-primary);
        }
    }

    &__bg-indicator {
        height: 6px;
        width: 100%;
        opacity: 0.8;
    }

    &__content {
        padding: var(--spacing-lg);
        display: flex;
        justify-content: space-between;
        align-items: center;
        flex-grow: 1;
    }

    &__badge-wrapper {
        margin-bottom: var(--spacing-sm);
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

    &__info {
        display: flex;
        flex-direction: column;
    }

    &__arrow {
        color: var(--color-text-muted);
        transition: all 0.2s ease;
    }
}
</style>
