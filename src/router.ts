import { markRaw } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";
import ShellLayout from "./layouts/ShellLayout.vue";
import IndexView from "./pages/IndexView.vue";
import InputsView from "./pages/InputsView.vue";
import LoginView from "./pages/LoginView.vue";
import ManageView from "./pages/ManageView.vue";
import RequestViewer from "./pages/RequestViewer.vue";
import SettingsView from "./pages/SettingsView.vue";
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
        path: "/manage",
        name: "manage",
        component: ManageView,
        meta: { layout: markRaw(ShellLayout) },
    },
    { path: "/manage/roms", redirect: "/manage" },
    { path: "/manage/emulators", redirect: "/manage" },
    {
        path: "/manage/roms/:console",
        name: "manage_roms_console",
        component: SystemStorageView,
        meta: { layout: markRaw(ShellLayout) },
    },
    {
        path: "/manage/inputs",
        name: "manage_inputs",
        component: InputsView,
        meta: { layout: markRaw(ShellLayout) },
    },
    {
        path: "/settings",
        name: "settings",
        component: SettingsView,
        meta: { layout: markRaw(ShellLayout) },
    },
    {
        path: "/dev/requests",
        name: "dev_requests",
        component: RequestViewer,
        meta: {},
    },
];

export const router = createRouter({
    history: createMemoryHistory(),
    routes,
});

router.beforeEach((to, _from, next) => {
    const authStore = useAuthStore();
    if (to.name !== "login" && to.name !== "dev_requests" && !authStore.token) {
        next({ name: "login" });
    } else {
        next();
    }
});

router.afterEach((_to, _from) => {
    const gameStore = useGameStore();
    gameStore.currentSelectedGame = null;
});
