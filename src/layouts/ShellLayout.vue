<script setup lang="ts">
import { ArrowLeftRight, HardDrive, Library, Menu, Search, Settings } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import Logout from "../components/Logout.vue";
import GameInfo from "../components/games/GameInfo.vue";
import ServerSwitcher from "../components/modals/ServerSwitcher.vue";
import { useAuthStore } from "../stores/AuthStore";
import { useGameStore } from "../stores/GameStore";
import { useUserStore } from "../stores/UserStore";

const userStore = useUserStore();
const authStore = useAuthStore();
const gameStore = useGameStore();
const isSidebarOpen = ref(false);
const isServerSwitcherOpen = ref(false);
const toggleSidebar = () => (isSidebarOpen.value = !isSidebarOpen.value);
const toggleServerSwitcher = () => (isServerSwitcherOpen.value = !isServerSwitcherOpen.value);
const ready = ref(false);

const menuItems = [
    { name: "Library", path: "/", icon: Library },
    { name: "Search", path: "/search", icon: Search },
    { name: "Storage", path: "/manage/roms", icon: HardDrive },
    { name: "Emulators", path: "/manage/emulators", icon: Settings },
];

const displayDomain = computed(() => {
    return (authStore?.domain || "").replace(/(^\w+:|^)\/\//, "").replace(/\/$/, "");
});

onMounted(() => {
    ready.value = true;
});
</script>

<template>
    <div class="c-shell">
        <Transition name="fade">
            <div v-if="isSidebarOpen" class="c-shell__sidebar-overlay" @click="toggleSidebar"></div>
        </Transition>

        <Transition name="slide-side">
            <aside v-if="isSidebarOpen" class="c-shell__sidebar">
                <nav class="c-shell__nav">
                    <router-link
                        v-for="item in menuItems"
                        :key="item.path"
                        :to="item.path"
                        class="c-shell__nav-link"
                        @click="toggleSidebar"
                    >
                        <div class="c-shell__nav-indicator"></div>
                        <component :is="item.icon" class="c-shell__nav-icon" />
                        <span class="c-shell__nav-text">{{ item.name }}</span>
                    </router-link>

                    <div class="c-shell__nav-spacer"></div>
                </nav>

                <div class="c-shell__sidebar-footer">
                    <button
                        class="c-shell__footer-btn"
                        title="Switch Server"
                        @click="
                            toggleServerSwitcher();
                            toggleSidebar();
                        "
                    >
                        <ArrowLeftRight class="c-shell__icon" />
                    </button>
                    <Logout class="c-shell__logout" />
                </div>
            </aside>
        </Transition>

        <header class="c-shell__status-bar">
            <button class="c-shell__menu-button" @click="toggleSidebar">
                <Menu class="c-shell__menu-icon" />
            </button>

            <div id="header-tools" class="c-shell__header-tools"></div>

            <div class="c-shell__system-meta">
                <span class="c-shell__domain-label">
                    {{ userStore.user?.username || "guest" }}@<a
                        class="c-shell__domain-link"
                        :href="authStore.domain || ''"
                        target="_blank"
                        >{{ displayDomain }}</a
                    >
                </span>
                <span class="c-shell__status-dot c-shell__status-dot--pulse"></span>
            </div>
        </header>

        <main class="c-shell__content">
            <slot v-if="ready" />
        </main>

        <ServerSwitcher :show="isServerSwitcherOpen" @close="isServerSwitcherOpen = false" />
        <div class="c-shell__info-wrapper" :class="{ 'c-shell__info-wrapper--visible': gameStore.currentSelectedGame }">
            <GameInfo v-if="gameStore.currentSelectedGame" />
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-shell {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: transparent;

    &__status-bar {
        position: relative;
        background: var(--glass-bg);
        backdrop-filter: blur(12px);
        -webkit-backdrop-filter: blur(12px);
        padding: 0 var(--spacing-md);
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 2px solid var(--color-border);
        z-index: 50;
        height: 56px;
        flex-shrink: 0;
    }

    &__menu-button {
        background: var(--color-surface);
        border: 2px solid var(--color-border);
        border-radius: var(--radius-sm);
        padding: var(--spacing-sm) var(--spacing-md);
        font-size: 0.75rem;
        font-weight: 800;
        color: var(--color-text);
        cursor: pointer;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
        transition: all 0.2s;
        letter-spacing: 1px;

        &:hover {
            border-color: var(--color-primary);
            transform: translateY(-1px);
        }
    }

    &__menu-icon {
        width: 24px;
        height: 24px;
        stroke-width: 2.5px;
        display: block;
    }

    &__nav {
        flex: 1;
        padding: var(--spacing-lg) var(--spacing-md);
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs);
    }

    &__nav-link {
        display: flex;
        align-items: center;
        padding: 12px var(--spacing-md);
        text-decoration: none;
        color: var(--color-text-muted);
        font-weight: 700;
        border-radius: var(--radius-md);
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        background: transparent;
        border: none;
        width: 100%;
        cursor: pointer;
        font-family: inherit;
        text-align: left;
        position: relative;
        overflow: hidden;

        &:hover {
            background: var(--color-surface-variant);
            color: var(--color-text);
        }

        &.router-link-active {
            background: var(--color-surface-variant);
            color: var(--color-primary);

            .c-shell__nav-indicator {
                transform: translateX(0);
            }

            .c-shell__nav-icon {
                color: var(--color-primary);
            }
        }
    }

    &__nav-icon {
        width: 20px;
        height: 20px;
        margin-right: var(--spacing-md);
        stroke-width: 2.5px;
        color: var(--color-text-muted);
    }

    &__nav-indicator {
        position: absolute;
        left: 0;
        top: 12px;
        bottom: 12px;
        width: 4px;
        background: var(--color-primary);
        border-radius: 0 4px 4px 0;
        transform: translateX(-4px);
        transition: transform 0.2s ease;
    }

    &__nav-spacer {
        flex: 1;
    }

    &__header-tools {
        flex: 1;
        display: flex;
        justify-content: center;
        align-items: center;
        height: 100%;
        padding: 0 var(--spacing-lg);
    }

    &__system-meta {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
    }

    &__domain-label {
        font-size: 0.8rem;
        font-weight: 800;
        color: var(--color-text-muted);
        letter-spacing: 0.5px;
    }

    &__domain-link {
        color: var(--color-primary);
        text-decoration: none;
        font-weight: 800;
    }

    &__status-dot {
        height: 10px;
        width: 10px;
        background-color: #4caf50;
        border-radius: var(--radius-full);
        box-shadow: 0 0 10px rgba(76, 175, 80, 0.4);
    }

    &__sidebar {
        position: fixed;
        top: var(--titlebar-height, 0);
        left: 0;
        bottom: 0;
        width: 280px;
        background: var(--color-surface);
        z-index: 1000;
        display: flex;
        flex-direction: column;
        box-shadow: 4px 0 20px rgba(0, 0, 0, 0.05);
        border-right: 1px solid var(--color-border);
    }

    &__sidebar-footer {
        padding: var(--spacing-md);
        border-top: 1px solid var(--color-border);
        background: var(--color-surface-variant);
        display: flex;
        gap: var(--spacing-sm);
        align-items: center;
    }

    &__footer-btn {
        width: 44px;
        height: 44px;
        border-radius: var(--radius-md);
        border: 2px solid var(--color-border);
        background: var(--color-surface);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: all 0.2s;
        color: var(--color-text-muted);

        &:hover {
            border-color: var(--color-primary);
            color: var(--color-primary);
            transform: translateY(-2px);
            background: white;
        }
    }

    &__icon {
        width: 24px;
        height: 24px;
        stroke-width: 2.5px;
    }

    &__logout {
        flex: 1;
    }

    &__sidebar-overlay {
        position: fixed;
        inset: 0;
        background: rgba(10, 10, 30, 0.4);
        backdrop-filter: blur(8px);
        z-index: 999;
    }

    &__content {
        flex: 1;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        min-height: 0;
    }

    &__info-wrapper {
        width: 100%;
        height: 0;
        overflow: hidden;
        transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
        background: var(--glass-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        z-index: 100;
        position: relative;
        flex-shrink: 0;

        &--visible {
            height: 120px;
            border-top: 1px solid var(--color-border);
            box-shadow: 0 -10px 30px rgba(0, 0, 0, 0.08);
        }
    }
}

.slide-side-enter-active,
.slide-side-leave-active {
    transition: transform 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-side-enter-from,
.slide-side-leave-to {
    transform: translateX(-100%);
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}
</style>
