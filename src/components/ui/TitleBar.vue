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
    <div data-tauri-drag-region class="titlebar">
        <div class="title-content" data-tauri-drag-region>
            <span class="app-logo">emuNEX</span>
            <span class="window-title">client</span>
        </div>
        <div class="window-controls">
            <button class="control-btn" @click="minimize" title="Minimize">
                <svg width="10" height="1" viewBox="0 0 10 1" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <line y1="0.5" x2="10" y2="0.5" stroke="currentColor" stroke-width="1"/>
                </svg>
            </button>
            <button class="control-btn" @click="toggleMaximize" title="Maximize">
                <svg width="10" height="10" viewBox="0 0 10 10" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor" stroke-width="1"/>
                </svg>
            </button>
            <button class="control-btn close-btn" @click="close" title="Close">
                <svg width="10" height="10" viewBox="0 0 10 10" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1" stroke-linecap="round"/>
                </svg>
            </button>
        </div>
    </div>
</template>

<style scoped>
.titlebar {
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
}

.title-content {
    display: flex;
    align-items: center;
    padding-left: var(--radius-md);
    flex-grow: 1;
    height: 100%;
}

.app-logo {
    font-weight: 950;
    color: var(--color-primary);
    font-size: 0.85rem;
    letter-spacing: -0.5px;
    margin-right: var(--spacing-xs);
}

.window-title {
    font-weight: 600;
    color: var(--color-text-muted);
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 1px;
}

.window-controls {
    display: flex;
    height: 100%;
    position: relative;
    z-index: 10000;
}

.control-btn {
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
}

.control-btn:hover {
    background: rgba(0, 0, 0, 0.05);
}

.close-btn:hover {
    background: var(--color-primary) !important;
    color: white !important;
}

.control-btn svg {
    display: block;
    shape-rendering: crispEdges;
}
</style>
