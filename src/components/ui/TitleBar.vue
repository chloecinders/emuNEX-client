<script setup lang="ts">
import { Window } from "@tauri-apps/api/window";
const appWindow = new Window("main");

const minimize = () => appWindow.minimize();
const toggleMaximize = async () => {
    const isMaximized = await appWindow.isMaximized();
    if (isMaximized) {
        appWindow.unmaximize();
    } else {
        appWindow.maximize();
    }
};
const close = () => appWindow.close();
</script>

<template>
    <div data-tauri-drag-region class="c-titlebar">
        <div class="c-titlebar__content" data-tauri-drag-region>
            <span class="c-titlebar__logo">emuNEX</span>
            <span class="c-titlebar__title">client</span>
        </div>
        <div class="c-titlebar__controls">
            <button class="c-titlebar__btn" @click="minimize" title="Minimize">
                <svg width="10" height="1" viewBox="0 0 10 1" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <line y1="0.5" x2="10" y2="0.5" stroke="currentColor" stroke-width="1"/>
                </svg>
            </button>
            <button class="c-titlebar__btn" @click="toggleMaximize" title="Maximize">
                <svg width="10" height="10" viewBox="0 0 10 10" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor" stroke-width="1"/>
                </svg>
            </button>
            <button class="c-titlebar__btn c-titlebar__btn--close" @click="close" title="Close">
                <svg width="10" height="10" viewBox="0 0 10 10" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1" stroke-linecap="round"/>
                </svg>
            </button>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-titlebar {
    height: var(--titlebar-height);
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    user-select: none;
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    z-index: 9999;

    &__content {
        display: flex;
        align-items: center;
        padding-left: var(--radius-md);
        flex-grow: 1;
        height: 100%;
    }

    &__logo {
        font-weight: 950;
        color: var(--color-primary);
        font-size: 0.85rem;
        letter-spacing: -0.5px;
        margin-right: var(--spacing-xs);
    }

    &__title {
        font-weight: 600;
        color: var(--color-text-muted);
        font-size: 0.7rem;
        text-transform: uppercase;
        letter-spacing: 1px;
    }

    &__controls {
        display: flex;
        height: 100%;
        position: relative;
        z-index: 10000;
    }

    &__btn {
        width: 44px;
        height: var(--titlebar-height);
        display: flex;
        align-items: center;
        justify-content: center;
        background: transparent;
        border: none;
        color: var(--color-text);
        cursor: pointer;
        transition: all 0.1s;

        &:hover {
            background: rgba(0, 0, 0, 0.05);
        }

        &--close:hover {
            background: var(--color-primary) !important;
            color: white !important;
        }

        svg {
            display: block;
            shape-rendering: crispEdges;
        }
    }
}
</style>
