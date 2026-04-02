<script setup lang="ts">
import { ArrowLeftRight, GamepadDirectional, HardDrive, Library, Settings } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useStoragePath } from "../lib/http";
import Logout from "../components/Logout.vue";
import GameInfo from "../components/games/GameInfo.vue";
import ServerSwitcher from "../components/modals/ServerSwitcher.vue";
import { useAuthStore } from "../stores/AuthStore";
import { useGameStore } from "../stores/GameStore";
import { useUpdateStore } from "../stores/UpdateStore";
import { useUserStore } from "../stores/UserStore";

const userStore = useUserStore();
const authStore = useAuthStore();
const gameStore = useGameStore();
const updateStore = useUpdateStore();
const router = useRouter();
const isServerSwitcherOpen = ref(false);
const toggleServerSwitcher = () => (isServerSwitcherOpen.value = !isServerSwitcherOpen.value);
const ready = ref(false);

const menuItems = [
    { name: "Library", path: "/", icon: Library },
    { name: "Storage", path: "/manage/roms", icon: HardDrive },
    { name: "Emulators", path: "/manage/emulators", icon: GamepadDirectional },
    { name: "Settings", path: "/settings", icon: Settings },
];

const displayDomain = computed(() => {
    return (authStore?.domain || "").replace(/(^\w+:|^)\/\//, "").replace(/\/$/, "");
});

const isProfileOpen = ref(false);
const toggleProfile = () => {
    isProfileOpen.value = !isProfileOpen.value;
};

const avatarUrl = computed(() => {
    if (!userStore.user?.avatar_path) return null;
    return useStoragePath(userStore.user.avatar_path);
});

const goToSettingsForUpdate = () => {
    updateStore.dismissBanner();
    router.push({ name: "settings" });
};

onMounted(() => {
    ready.value = true;
});
</script>

<template>
    <div class="c-shell">
        <header class="c-shell__status-bar">
            <div id="header-tools" class="c-shell__header-tools"></div>

            <nav class="c-shell__top-nav">
                <router-link v-for="item in menuItems" :key="item.path" :to="item.path" class="c-shell__top-nav-link">
                    <component :is="item.icon" class="c-shell__top-nav-icon" />
                    <span class="c-shell__top-nav-text">{{ item.name }}</span>
                </router-link>

                <div class="c-shell__nav-spacer"></div>

                <div class="c-shell__user-meta">
                    <button class="c-shell__top-btn" title="Switch Server" @click="toggleServerSwitcher">
                        <ArrowLeftRight class="c-shell__btn-icon" />
                    </button>

                    <div class="c-shell__profile-dropdown-wrapper" @mouseleave="isProfileOpen = false">
                        <button class="c-shell__avatar-btn" @click="toggleProfile">
                            <div class="c-shell__avatar-placeholder">
                                <template v-if="avatarUrl">
                                    <img :src="avatarUrl" class="c-shell__avatar-img" />
                                </template>
                                <template v-else>
                                    {{ (userStore.user?.username || "G").charAt(0).toUpperCase() }}
                                </template>
                            </div>
                        </button>

                        <Transition name="fade">
                            <div v-show="isProfileOpen" class="c-shell__profile-menu-wrap">
                                <div class="c-shell__profile-menu">
                                    <div class="c-shell__profile-header">
                                        <div class="c-shell__avatar-placeholder c-shell__avatar-placeholder--large">
                                            <template v-if="avatarUrl">
                                                <img :src="avatarUrl" class="c-shell__avatar-img" />
                                            </template>
                                            <template v-else>
                                                {{ (userStore.user?.username || "G").charAt(0).toUpperCase() }}
                                            </template>
                                        </div>
                                        <div class="c-shell__user-info">
                                            <span class="c-shell__username">{{
                                                userStore.user?.username || "guest"
                                            }}</span>
                                            <a
                                                class="c-shell__domain-link"
                                                :href="authStore.domain || ''"
                                                target="_blank"
                                            >
                                                @{{ displayDomain }}
                                            </a>
                                        </div>
                                    </div>
                                    <div class="c-shell__profile-actions">
                                        <Logout class="c-shell__logout-btn" />
                                    </div>
                                </div>
                            </div>
                        </Transition>
                    </div>
                </div>
            </nav>
        </header>

        <Transition name="fade">
            <div v-if="updateStore.hasUpdate && !updateStore.bannerDismissed" class="c-shell__update-banner">
                <span class="c-shell__update-text">
                    Update available<span v-if="updateStore.availableVersion">
                        - v{{ updateStore.availableVersion }}</span
                    >
                </span>
                <button class="c-shell__update-button" @click="goToSettingsForUpdate"> Open Settings </button>
                <button class="c-shell__update-dismiss" @click="updateStore.dismissBanner"> Dismiss </button>
            </div>
        </Transition>

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
        padding: var(--spacing-sm) var(--spacing-md);
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 2px solid var(--color-border);
        z-index: 50;
        min-height: 56px;
        flex-shrink: 0;
        flex-wrap: wrap-reverse;
        gap: var(--spacing-md);
    }

    &__header-tools {
        flex: 10000 1 auto;
        min-width: 250px;
        display: flex;
        justify-content: flex-start;
        align-items: center;
    }

    &__top-nav {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        flex: 1 1 auto;
        justify-content: flex-start;
    }

    &__top-nav-link {
        display: flex;
        align-items: center;
        padding: var(--spacing-xs) var(--spacing-md);
        text-decoration: none;
        color: var(--color-text-muted);
        font-weight: 700;
        border-radius: var(--radius-full);
        transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
        background: transparent;
        border: 1px solid transparent;
        white-space: nowrap;

        &:hover {
            color: var(--color-text);
            background: rgba(255, 255, 255, 0.05); // Or any hover surface
            border-color: var(--color-border);
        }

        &.router-link-active {
            background: var(--color-primary);
            color: white;
            border-color: var(--color-primary);

            .c-shell__top-nav-icon {
                color: white;
            }
        }
    }

    &__top-nav-icon {
        width: 16px;
        height: 16px;
        margin-right: var(--spacing-sm);
        stroke-width: 2.5px;
    }

    &__nav-spacer {
        flex: 1;
    }

    &__user-meta {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
    }

    &__top-btn {
        width: 32px;
        height: 32px;
        border-radius: var(--radius-full);
        border: 1px solid var(--color-border);
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
        }
    }

    &__btn-icon {
        width: 16px;
        height: 16px;
        stroke-width: 2.5px;
    }

    &__profile-dropdown-wrapper {
        position: relative;
        display: flex;
        align-items: center;
    }

    &__avatar-btn {
        background: transparent;
        border: none;
        padding: 0;
        cursor: pointer;
        outline: none;
        border-radius: var(--radius-full);
        transition: transform 0.2s cubic-bezier(0.175, 0.885, 0.32, 1);

        &:hover {
            transform: scale(1.05);
        }
    }

    &__avatar-placeholder {
        width: 32px;
        height: 32px;
        border-radius: var(--radius-full);
        background: var(--color-primary);
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 800;
        font-size: 1rem;
        flex-shrink: 0;
        box-shadow: 0 2px 5px rgba(107, 92, 177, 0.3);

        &--large {
            width: 48px;
            height: 48px;
            font-size: 1.5rem;
        }
    }

    &__avatar-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        border-radius: inherit;
    }

    &__profile-menu-wrap {
        position: absolute;
        top: 100%;
        right: 0;
        padding-top: var(--spacing-sm);
        z-index: 1000;
    }

    &__profile-menu {
        width: 260px;
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-lg);
        box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    &__profile-header {
        padding: var(--spacing-lg);
        background: var(--color-surface-variant);
        border-bottom: 1px solid var(--color-border);
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
    }

    &__user-info {
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    &__username {
        font-size: 1.1rem;
        font-weight: 800;
        color: var(--color-text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    &__domain-link {
        font-size: 0.8rem;
        color: var(--color-text-muted);
        text-decoration: none;
        font-weight: 700;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        transition: color 0.15s;

        &:hover {
            color: var(--color-primary);
        }
    }

    &__profile-actions {
        padding: var(--spacing-md);
        display: flex;
    }

    &__logout-btn {
        width: 100%;
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
    transform: translateX(100%);
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
