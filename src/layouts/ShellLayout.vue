<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import Logout from "../components/Logout.vue";
import ServerSwitcher from "../components/modals/ServerSwitcher.vue";
import { useAuthStore } from "../stores/AuthStore";
import { useUserStore } from "../stores/UserStore";
import { Menu, ArrowLeftRight, Library, Search } from "lucide-vue-next";

const userStore = useUserStore();
const authStore = useAuthStore();
const isSidebarOpen = ref(false);
const isServerSwitcherOpen = ref(false);
const toggleSidebar = () => (isSidebarOpen.value = !isSidebarOpen.value);
const toggleServerSwitcher = () => (isServerSwitcherOpen.value = !isServerSwitcherOpen.value);
const ready = ref(false);

const menuItems = [
    { name: "Library", path: "/", icon: Library },
    { name: "Search", path: "/search", icon: Search },
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
                <nav class="sidebar-content">
                    <router-link
                        v-for="item in menuItems"
                        :key="item.path"
                        :to="item.path"
                        class="nav-link"
                        @click="toggleSidebar"
                    >
                        <div class="nav-indicator"></div>
                        <component :is="item.icon" class="nav-icon" />
                        <span class="nav-text">{{ item.name }}</span>
                    </router-link>

                    <div class="nav-spacer"></div>
                </nav>

                <div class="sidebar-footer">
                    <button
                        class="footer-btn server-btn"
                        title="Switch Server"
                        @click="
                            toggleServerSwitcher();
                            toggleSidebar();
                        "
                    >
                        <ArrowLeftRight class="icon" />
                    </button>
                    <Logout class="logout-wrapper" />
                </div>
            </aside>
        </Transition>

        <header class="status-bar">
            <button class="menu-button" @click="toggleSidebar">
                <Menu class="menu-icon" />
            </button>

            <div id="header-tools" class="header-tools-container"></div>

            <div class="system-meta">
                <span class="domain-label">
                    {{ userStore.user?.username || "guest" }}@{{ displayDomain }}
                </span>
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
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: transparent;
}

.status-bar {
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

.menu-button {
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
}

.menu-button:hover {
    border-color: var(--color-primary);
    transform: translateY(-1px);
}

.menu-icon {
    width: 24px;
    height: 24px;
    stroke-width: 2.5px;
    display: block;
}

.nav-icon {
    width: 20px;
    height: 20px;
    margin-right: var(--spacing-md);
    stroke-width: 2.5px;
    color: var(--color-text-muted);
}

.router-link-active .nav-icon {
    color: var(--color-primary);
}

.header-tools-container {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    padding: 0 var(--spacing-lg);
}

.system-meta {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
}

.domain-label {
    font-size: 0.8rem;
    font-weight: 800;
    color: var(--color-text-muted);
    letter-spacing: 0.5px;
}


.status-dot {
    height: 10px;
    width: 10px;
    background-color: #4caf50;
    border-radius: var(--radius-full);
    box-shadow: 0 0 10px rgba(76, 175, 80, 0.4);
}

.sidebar {
    position: fixed;
    top: var(--titlebar-height);
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

.sidebar-content {
    flex: 1;
    padding: var(--spacing-lg) var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-xs);
}

.nav-link {
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
}

.nav-link:hover {
    background: var(--color-surface-variant);
    color: var(--color-text);
}

.nav-link.router-link-active {
    background: var(--color-surface-variant);
    color: var(--color-primary);
}

.nav-indicator {
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

.router-link-active .nav-indicator {
    transform: translateX(0);
}

.nav-spacer {
    flex: 1;
}

.server-switch-btn {
    margin-top: auto;
    border: 2px dashed var(--color-border);
    justify-content: center;
}

.server-switch-btn:hover {
    border-color: var(--color-primary);
    background: var(--color-surface-variant);
}

.sidebar-footer {
    padding: var(--spacing-md);
    border-top: 1px solid var(--color-border);
    background: var(--color-surface-variant);
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
}

.footer-btn {
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
}

.footer-btn:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
    transform: translateY(-2px);
    background: white;
}

.icon {
    width: 24px;
    height: 24px;
    stroke-width: 2.5px;
}

.logout-wrapper {
    flex: 1;
}

.sidebar-overlay {
    position: fixed;
    inset: 0;
    background: rgba(10, 10, 30, 0.4);
    backdrop-filter: blur(8px);
    z-index: 999;
}

.page-content {
    flex-grow: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
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
