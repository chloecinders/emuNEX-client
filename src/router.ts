import { markRaw } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";
import ShellLayout from "./layouts/ShellLayout.vue";
import EmulatorView from "./pages/EmulatorView.vue";
import IndexView from "./pages/IndexView.vue";
import LoginView from "./pages/LoginView.vue";

import SettingsView from "./pages/SettingsView.vue";
import StorageView from "./pages/StorageView.vue";
import SystemStorageView from "./pages/SystemStorageView.vue";
import { useAuthStore } from "./stores/AuthStore";
import { useGameStore } from "./stores/GameStore";

const routes = [
    {
        path: "/login",
        name: "login",
        component: LoginView,
        meta: {},
    },
    {
        path: "/",
        name: "home",
        component: IndexView,
        meta: { layout: markRaw(ShellLayout) },
    },

    {
        path: "/manage/emulators",
        name: "manage_emulators",
        component: EmulatorView,
        meta: { layout: markRaw(ShellLayout) },
    },
    {
        path: "/manage/roms",
        name: "manage_roms",
        component: StorageView,
        meta: { layout: markRaw(ShellLayout) },
    },
    {
        path: "/manage/roms/:console",
        name: "manage_roms_console",
        component: SystemStorageView,
        meta: { layout: markRaw(ShellLayout) },
    },
    {
        path: "/settings",
        name: "settings",
        component: SettingsView,
        meta: { layout: markRaw(ShellLayout) },
    }
];

export const router = createRouter({
    history: createMemoryHistory(),
    routes,
});

router.beforeEach((to, _from, next) => {
    const authStore = useAuthStore();
    if (to.name !== "login" && !authStore.token) {
        next({ name: "login" });
    } else {
        next();
    }
});

router.afterEach((_to, _from) => {
    const gameStore = useGameStore();
    gameStore.currentSelectedGame = null;
});
