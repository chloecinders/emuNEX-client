<script setup lang="ts">
import { ChevronDown, ChevronUp, FolderOpen, Trash2 } from "lucide-vue-next";
import { ref } from "vue";
import { formatBytes } from "../../lib/format";
import { useRomStore, type GameSaveFile, type LocalStorageEntry } from "../../stores/RomStore";
import Button from "../ui/Button.vue";
import Heading from "../ui/Heading.vue";
import Spinner from "../ui/Spinner.vue";
import Text from "../ui/Text.vue";

const props = defineProps<{
    rom: LocalStorageEntry;
    gameTitle: string;
    cloudVersion?: number;
}>();

const emit = defineEmits<{
    (e: "deleteRom"): void;
    (e: "deleteSave"): void;
}>();

const romStore = useRomStore();

const isExpanded = ref(false);
const saveFiles = ref<GameSaveFile[]>([]);
const isLoadingSaves = ref(false);

const toggleSaveDetails = async () => {
    isExpanded.value = !isExpanded.value;
    if (isExpanded.value && saveFiles.value.length === 0) {
        isLoadingSaves.value = true;
        saveFiles.value = await romStore.getGameSaveFiles(props.rom.game_id);
        isLoadingSaves.value = false;
    }
};
</script>

<template>
    <div class="c-rom-card" tabindex="0" role="button">
        <div class="c-rom-card__info">
            <Heading :level="3" class="c-rom-card__title">{{ gameTitle }}</Heading>
            <div class="c-rom-card__meta">
                <Text v-if="rom.rom_size > 0" size="sm" variant="muted"
                    ><strong>ROM:</strong> {{ formatBytes(rom.rom_size) }}</Text
                >
                <Text v-else size="sm" color="red"><strong>ROM Missing</strong></Text>
                <Text v-if="rom.save_size > 0" size="sm" variant="muted"
                    >• <strong>Save:</strong> {{ formatBytes(rom.save_size) }}</Text
                >
                <Text
                    v-if="rom.local_version !== undefined && rom.local_version !== null"
                    size="sm"
                    variant="muted"
                    >• <strong>Local:</strong> v{{ rom.local_version }}</Text
                >
                <Text v-if="cloudVersion !== undefined" size="sm" variant="muted"
                    >• <strong>Cloud:</strong> v{{ cloudVersion }}</Text
                >
            </div>

            <button v-if="rom.save_size > 0" class="c-rom-card__toggle-link" @click="toggleSaveDetails">
                <template v-if="isExpanded"><ChevronUp :size="14" /> Hide save files</template>
                <template v-else><ChevronDown :size="14" /> Show save files</template>
            </button>
        </div>

        <div v-if="isExpanded" class="c-rom-card__save-details">
            <div v-if="isLoadingSaves" class="c-rom-card__loading-saves">
                <Spinner size="sm" /> <Text size="sm">Loading files...</Text>
            </div>
            <div v-else-if="saveFiles.length === 0" class="c-rom-card__no-saves">
                <Text variant="muted" size="sm">No save files found.</Text>
            </div>
            <ul v-else class="c-rom-card__file-list">
                <li v-for="file in saveFiles" :key="file.path">
                    <div class="c-rom-card__file-info">
                        <Text size="sm" class="c-rom-card__file-path">{{ file.path }}</Text>
                    </div>
                    <Text size="sm" variant="muted">{{ formatBytes(file.size) }}</Text>
                </li>
            </ul>
        </div>

        <div class="c-rom-card__actions">
            <Button v-if="rom.save_size > 0" color="grey" size="sm" @click="romStore.openSaveFolder(rom.game_id)">
                <FolderOpen :size="16" /> Open
            </Button>
            <Button v-if="rom.save_size > 0" color="grey" size="sm" @click="emit('deleteSave')">
                <Trash2 :size="16" /> Delete Save
            </Button>
            <Button v-if="rom.rom_size > 0" color="red" size="sm" @click="emit('deleteRom')">
                <Trash2 :size="16" /> Delete ROM
            </Button>
        </div>
    </div>
</template>

<style lang="scss" scoped>
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
        gap: var(--spacing-xs);
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

    &__toggle-link {
        background: none;
        border: none;
        color: var(--color-primary);
        font-family: inherit;
        font-size: 0.85rem;
        cursor: pointer;
        padding: 0;
        display: inline-flex;
        align-items: center;
        gap: var(--spacing-xs);
        align-self: flex-start;
        font-weight: 600;

        &:hover {
            color: var(--color-primary-hover);
            text-decoration: underline;
        }
    }

    &__save-details {
        border-top: 1px solid var(--color-border);
        padding-top: var(--spacing-md);
    }

    &__loading-saves,
    &__no-saves {
        display: flex;
        justify-content: center;
        padding: var(--spacing-sm) 0;
        gap: var(--spacing-sm);
        align-items: center;
    }

    &__file-list {
        list-style: none;
        padding: 0;
        margin: 0;
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs);

        li {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: var(--spacing-xs) var(--spacing-sm);
            background: rgba(255, 255, 255, 0.03);
            border-radius: var(--radius-sm);
            gap: var(--spacing-md);
        }
    }

    &__file-info {
        display: flex;
        flex-direction: column;
        flex: 1;
        gap: 2px;
    }

    &__file-path {
        font-family: monospace;
        word-break: break-all;
    }
}
</style>
