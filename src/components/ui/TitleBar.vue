<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { CircleHelp } from "lucide-vue-next";
import { ref } from "vue";
import { openExternalUrl } from "../../lib/opener";
const appWindow = getCurrentWindow();
const isRequestsWindow = appWindow.label === "requests";

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

const helpMenu = ref({ show: false, x: 0, y: 0 });
const closeHelpMenu = () => (helpMenu.value.show = false);
const onHelpClick = (event: MouseEvent) => {
    event.stopPropagation();
    helpMenu.value = { show: true, x: event.clientX, y: event.clientY };
    document.addEventListener("click", closeHelpMenu, { once: true });
};

const bugReportUrl = "https://github.com/chloecinders/emuNEX-client/issues/new?template=bug_report.md";
const featureRequestUrl = "https://github.com/chloecinders/emuNEX-client/issues/new?template=feature_request.md";
const discordUrl = "https://discord.gg/uF65zSUnRs";

const openIssueLink = async (url: string) => {
    helpMenu.value.show = false;
    await openExternalUrl(url);
};
</script>

<template>
    <div data-tauri-drag-region class="c-titlebar">
        <div class="c-titlebar__content" data-tauri-drag-region>
            <span class="c-titlebar__logo">emuNEX</span>
            <span class="c-titlebar__title">{{ isRequestsWindow ? "Request Viewer" : "client" }}</span>
        </div>
        <div class="c-titlebar__controls">
            <button class="c-titlebar__btn" @click="onHelpClick" title="Help">
                <CircleHelp class="c-titlebar__lucide-icon" :size="16" />
            </button>
            <button class="c-titlebar__btn" @click="minimize" title="Minimize">
                <svg
                    class="c-titlebar__control-svg"
                    width="10"
                    height="1"
                    viewBox="0 0 10 1"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <line y1="0.5" x2="10" y2="0.5" stroke="currentColor" stroke-width="1" />
                </svg>
            </button>
            <button class="c-titlebar__btn" @click="toggleMaximize" title="Maximize">
                <svg
                    class="c-titlebar__control-svg"
                    width="10"
                    height="10"
                    viewBox="0 0 10 10"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor" stroke-width="1" />
                </svg>
            </button>
            <button class="c-titlebar__btn c-titlebar__btn--close" @click="close" title="Close">
                <svg
                    class="c-titlebar__control-svg"
                    width="10"
                    height="10"
                    viewBox="0 0 10 10"
                    fill="none"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
                </svg>
            </button>
        </div>

        <div
            v-if="helpMenu.show"
            class="c-titlebar__help-menu"
            :style="{ left: helpMenu.x + 'px', top: helpMenu.y + 'px' }"
            @click.stop
        >
            <button class="c-titlebar__help-item" @click="openIssueLink(discordUrl)">Join the Discord...</button>
            <button class="c-titlebar__help-item" @click="openIssueLink(bugReportUrl)">Report a bug...</button>
            <button class="c-titlebar__help-item" @click="openIssueLink(featureRequestUrl)"
                >Suggest a feature...</button
            >
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
        pointer-events: none;
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
        }
    }

    &__control-svg {
        shape-rendering: crispEdges;
    }

    &__lucide-icon {
        display: block;
        shape-rendering: geometricPrecision;
    }

    &__help-menu {
        position: fixed;
        z-index: 10001;
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-sm);
        box-shadow: var(--shadow-lg);
        padding: var(--spacing-xxs) 0;
        min-width: 220px;
        transform: translate(-100%, 10px);
    }

    &__help-item {
        display: block;
        width: 100%;
        text-align: left;
        background: transparent;
        border: none;
        padding: var(--spacing-sm) var(--spacing-md);
        color: var(--color-text);
        cursor: pointer;
        font-family: inherit;
        font-size: 0.9rem;

        &:hover {
            background: var(--color-surface-variant);
        }
    }
}
</style>
