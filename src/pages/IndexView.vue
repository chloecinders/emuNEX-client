<script setup lang="ts">
import { inject, Ref, ref, watch } from "vue";
import GameInfo from "../components/games/GameInfo.vue";
import GameLibrary from "../components/games/GameLibrary.vue";
import { router } from "../router";
import { useAuthStore } from "../stores/AuthStore";
import { LibraryGame, useGameStore } from "../stores/GameStore";
import { useUserStore } from "../stores/UserStore";

const authStore = useAuthStore();
const userStore = useUserStore();
const gameStore = useGameStore();
const library: Ref<LibraryGame[]> = ref([]);

if (!authStore.domain || !authStore.token) {
    const token = inject<string>("auth_token");
    const domain = inject<string>("auth_domain");

    if (token && domain) {
        authStore.token = token;
        authStore.domain = domain;
    } else {
        router.push("/login");
    }
}

gameStore.fetchLibrary();

watch(
    () => gameStore.library,
    (newLibrary: LibraryGame[]) => {
        library.value = newLibrary;
    },
    { immediate: true },
);
</script>

<template>
    <Teleport to="#header-tools">
        <div class="user-pill">
            <span class="username">Welcome, {{ userStore.user?.username }}!</span>
        </div>
    </Teleport>

    <div class="library-container">
        <h1 class="section-title">Software Library</h1>

        <GameLibrary :games="library" />
    </div>

    <GameInfo />
</template>

<style scoped>
.library-container {
    padding: 20px;
}

.section-title {
    font-size: 1.5rem;
    font-weight: 900;
    margin-bottom: 20px;
    color: var(--3ds-text);
    text-transform: uppercase;
    letter-spacing: 1px;
}

.user-pill {
    display: flex;
    align-items: center;
    gap: 8px;
}

.username {
    font-weight: 800;
    font-size: 0.9rem;
}
</style>
