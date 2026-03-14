import { markRaw } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";
import ShellLayout from "./layouts/ShellLayout.vue";
import IndexView from "./pages/IndexView.vue";
import LoginView from "./pages/LoginView.vue";
import SearchView from "./pages/SearchView.vue";
import { useGameStore } from "./stores/GameStore";

const routes = [
    {
        path: '/login',
        component: LoginView,
        meta: {}
    },
    {
        path: '/',
        component: IndexView,
        meta: { layout: markRaw(ShellLayout) }
    },
    {
        path: '/search',
        component: SearchView,
        meta: { layout: markRaw(ShellLayout) }
    }
];

export const router = createRouter({
    history: createMemoryHistory(),
    routes,
})

router.afterEach((_to, _from) => {
    const gameStore = useGameStore();
    gameStore.currentSelectedGame = null;
})
