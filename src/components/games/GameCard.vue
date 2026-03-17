<script setup lang="ts">
import { useStoragePath } from "../../lib/http";
import { useConsoleStore } from "../../stores/ConsoleStore";
import { type PartialGame } from "../../stores/GameStore";

defineProps<{
    game: PartialGame;
    isDragging?: boolean;
    isInsertBefore?: boolean;
}>();

const emit = defineEmits(["mousedown", "click"]);
const consoleStore = useConsoleStore();
</script>

<template>
    <div
        class="c-game-card"
        :class="{
            'c-game-card--dragging': isDragging,
            'c-game-card--insert-before': isInsertBefore,
        }"
        :data-game-id="game.id"
        :style="{ background: consoleStore.getConsoleColor(game.console) }"
        @mousedown="emit('mousedown', $event, game.id)"
        @click="emit('click', game.id)"
    >
        <div
            v-if="game.region"
            class="c-game-card__region"
            :style="{ background: consoleStore.getConsoleColor(game.console) }"
        >
            {{ game.region }}
        </div>

        <img :src="useStoragePath(game.image_path)" :alt="game.title" class="c-game-card__cover" />
    </div>
</template>

<style lang="scss" scoped>
.c-game-card {
    position: relative;
    aspect-ratio: 6/9;
    width: 100%;
    border-radius: var(--radius-md);
    overflow: hidden;
    box-shadow: var(--shadow-subtle);
    cursor: pointer;
    transition: all 0.25s cubic-bezier(0.175, 0.885, 0.32, 1.275);
    background: var(--color-surface);
    border: var(--spacing-xxs) solid transparent;
    -webkit-user-drag: element;

    &:hover {
        transform: scale(1.08) translateY(-var(--spacing-xs));
        z-index: 10;
        box-shadow: var(--shadow-md);
        border-color: var(--color-primary-light);
    }

    &:active {
        transform: scale(0.95);
    }

    &--dragging {
        opacity: 0.35;
        transform: scale(0.95);
    }

    &--insert-before {
        position: relative;

        &::before {
            content: "";
            position: absolute;
            left: -6px;
            top: 0;
            bottom: 0;
            width: 3px;
            background: var(--color-primary);
            border-radius: 99px;
            box-shadow: 0 0 8px var(--color-primary);
            z-index: 10;
        }
    }

    &__region {
        position: absolute;
        top: 0;
        right: 0;
        font-weight: 800;
        font-size: 0.65rem;
        padding: var(--spacing-xxs) var(--spacing-xs);
        border-bottom-left-radius: var(--radius-md);
    }

    &__cover {
        width: 100%;
        height: 100%;
        object-fit: cover;
        -webkit-user-drag: none;
    }

    * {
        pointer-events: none;
    }
}
</style>
