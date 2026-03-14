import { markRaw } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";
import ShellLayout from "./layouts/ShellLayout.vue";
import IndexView from "./pages/IndexView.vue";
import LoginView from "./pages/LoginView.vue";
import SearchView from "./pages/SearchView.vue";
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
        path: "/search",
        name: "search",
        component: SearchView,
        meta: { layout: markRaw(ShellLayout) },
    },
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
