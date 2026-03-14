<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import Logout from "../components/Logout.vue";
import ServerSwitcher from "../components/modals/ServerSwitcher.vue";
import { useAuthStore } from "../stores/AuthStore";
import { useUserStore } from "../stores/UserStore";

const userStore = useUserStore();
const authStore = useAuthStore();
const isSidebarOpen = ref(false);
const isServerSwitcherOpen = ref(false);
const toggleSidebar = () => (isSidebarOpen.value = !isSidebarOpen.value);
const toggleServerSwitcher = () => (isServerSwitcherOpen.value = !isServerSwitcherOpen.value);
const ready = ref(false);

const menuItems = [
    { name: "Library", path: "/" },
    { name: "Search", path: "/search" },
];

const displayDomain = computed(() => {
    return (authStore?.domain || "").replace(/(^\w+:|^)\/\//, "").replace(/\/$/, "");
});

onMounted(() => {
    ready.value = true;
});
</script>

<template>
    <div class="nintendo-shell">
        <Transition name="fade">
            <div v-if="isSidebarOpen" class="sidebar-overlay" @click="toggleSidebar"></div>
        </Transition>

        <Transition name="slide-side">
            <aside v-if="isSidebarOpen" class="sidebar">
                <div class="sidebar-header">
                    <div class="mii-container">
                        <div class="mii-avatar"></div>
                    </div>
                    <div class="user-details">
                        <span class="username-label">{{ userStore.user?.username || "Guest" }}</span>
                        <span class="online-status">Online</span>
                    </div>
                </div>

                <nav class="sidebar-content">
                    <router-link
                        v-for="item in menuItems"
                        :key="item.path"
                        :to="item.path"
                        class="nav-link"
                        @click="toggleSidebar"
                    >
                        <div class="nav-indicator"></div>
                        <span class="nav-text">{{ item.name }}</span>
                    </router-link>

                    <div class="nav-spacer"></div>

                    <button class="nav-link server-switch-btn" @click="toggleServerSwitcher(); toggleSidebar()">
                        <div class="nav-indicator"></div>
                        <span class="nav-text">Switch Server</span>
                    </button>
                </nav>

                <div class="sidebar-footer">
                    <Logout class="logout-wrapper" />
                </div>
            </aside>
        </Transition>

        <header class="status-bar">
            <div class="glass-glare"></div>
            <button class="menu-button" @click="toggleSidebar">MENU</button>

            <div id="header-tools" class="header-tools-container"></div>

            <div class="system-meta">
                <span class="domain-label">{{ displayDomain }}</span>
                <span class="status-dot pulse"></span>
            </div>
        </header>

        <main class="page-content">
            <slot v-if="ready" />
        </main>

        <ServerSwitcher :show="isServerSwitcherOpen" @close="isServerSwitcherOpen = false" />
    </div>
</template>

<style scoped>
.nintendo-shell {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.status-bar {
    position: relative;
    background: linear-gradient(to bottom, #ffffff 0%, #f1f1f1 50%, #e2e2e2 100%);
    padding: 0 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 3px solid var(--3ds-border);
    z-index: 50;
    height: 60px;
    flex-shrink: 0;
}

.glass-glare {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 50%;
    background: linear-gradient(rgba(255, 255, 255, 0.8), rgba(255, 255, 255, 0));
    pointer-events: none;
}

.header-left {
    display: flex;
    align-items: center;
    gap: 20px;
    flex: 1;
    height: 100%;
}

.menu-button {
    background: white;
    border: 2px solid var(--3ds-border);
    border-radius: 8px;
    padding: 6px 12px;
    font-size: 0.7rem;
    font-weight: 800;
    color: var(--3ds-text);
    cursor: pointer;
    box-shadow: 0 2px 0 #bbb;
}

.system-meta {
    display: flex;
    align-items: center;
    gap: 10px;
}
.domain-label {
    font-size: 0.7rem;
    font-weight: bold;
    color: #888;
    text-transform: uppercase;
}

.status-dot {
    height: 8px;
    width: 8px;
    background-color: #4caf50;
    border-radius: 50%;
    box-shadow: 0 0 4px #4caf50;
}

.pulse {
    animation: nintendo-pulse 2s infinite ease-in-out;
}
@keyframes nintendo-pulse {
    0%,
    100% {
        opacity: 1;
    }
    50% {
        opacity: 0.4;
    }
}

.page-content {
    padding: 32px;
    flex-grow: 1;
    overflow-y: auto;
}

.sidebar {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: 300px;
    background: #ffffff;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    box-shadow: 15px 0 40px rgba(0, 0, 0, 0.15);
    border-right: 8px solid var(--3ds-red, #0072c6);
}

.sidebar-header {
    padding: 24px 24px 24px 24px;
    display: flex;
    align-items: center;
    gap: 16px;
    background: linear-gradient(to bottom, #fcfcfc, #f5f5f5);
    border-bottom: 2px solid #eee;
}

.mii-avatar {
    width: 52px;
    height: 52px;
    background: #ff7e12;
    border-radius: 50%;
    border: 3px solid #fff;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.username-label {
    display: block;
    font-size: 1.2rem;
    font-weight: 900;
    color: #333;
}

.online-status {
    font-size: 0.75rem;
    color: #4caf50;
    font-weight: bold;
    text-transform: uppercase;
}

.sidebar-content {
    flex: 1;
    padding: 24px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.nav-link {
    display: flex;
    align-items: center;
    padding: 14px 20px;
    text-decoration: none;
    color: #555;
    font-weight: 700;
    border-radius: 12px;
    transition: all 0.2s ease;
    position: relative;
    overflow: hidden;
    background: transparent;
    border: none;
    width: 100%;
    cursor: pointer;
    font-family: inherit;
    text-align: left;
}

.nav-spacer {
    flex: 1;
}

.server-switch-btn {
    margin-top: auto;
    border: 2px solid #eee;
}

.nav-link:hover {
    background: #f0f0f0;
    color: #000;
}

.nav-link.router-link-active {
    background: var(--3ds-blue, #0072c6);
    color: #fff;
    box-shadow: 0 4px 12px rgba(0, 114, 198, 0.3);
}

.nav-indicator {
    width: 4px;
    height: 20px;
    background: transparent;
    margin-right: 12px;
    border-radius: 2px;
}

.router-link-active .nav-indicator {
    background: #fff;
}

.sidebar-footer {
    padding: 24px;
    border-top: 2px solid #eee;
    background: #fafafa;
}

.logout-wrapper {
    width: 100%;
}

.sidebar-overlay {
    position: fixed;
    inset: 0;
    background: rgba(40, 40, 45, 0.5);
    backdrop-filter: blur(4px);
    z-index: 999;
}

.slide-side-enter-active,
.slide-side-leave-active {
    transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
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
