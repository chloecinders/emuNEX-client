<script setup lang="ts">
import { computed, watch } from "vue";
import { useRoute } from "vue-router";
import Button from "./components/ui/Button.vue";
import Heading from "./components/ui/Heading.vue";
import Spinner from "./components/ui/Spinner.vue";
import Text from "./components/ui/Text.vue";
import TitleBar from "./components/ui/TitleBar.vue";
import DefaultLayout from "./layouts/DefaultLayout.vue";
import { router } from "./router";
import { useAuthStore } from "./stores/AuthStore";
import { useGameStore } from "./stores/GameStore";

const route = useRoute();
const gameStore = useGameStore();
const authStore = useAuthStore();

const layout = computed(() => {
    return route.meta.layout || DefaultLayout;
});

watch(
    () => authStore.token,
    (newToken) => {
        if (newToken && route.name === "login") {
            router.push("/");
        }
    },
);
</script>

<template>
    <TitleBar />

    <div class="c-app-main">
        <transition name="dim">
            <div v-if="authStore.isConnecting && route.name && route.name !== 'login'" class="c-app-dimmer">
                <div class="c-dimmer-content">
                    <Spinner size="lg" />
                    <Heading :level="3" style="margin-top: var(--spacing-lg)">Connecting...</Heading>
                    <Text variant="muted" style="display: block">Checking connection to server</Text>
                </div>
            </div>
            <div v-else-if="authStore.connectionError && route.name && route.name !== 'login'" class="c-app-dimmer">
                <div class="c-dimmer-content">
                    <Heading :level="3">Connection Error</Heading>
                    <Text variant="error" style="display: block; margin: var(--spacing-md) 0 var(--spacing-lg)">
                        {{ authStore.connectionError }}
                    </Text>

                    <div style="display: flex; gap: var(--spacing-md); justify-content: center">
                        <Button @click="authStore.startup()"> Retry Connection </Button>
                        <Button variant="outline" @click="authStore.logout()"> Switch Server </Button>
                    </div>
                </div>
            </div>
            <div v-else-if="gameStore.isDimmed" class="c-app-dimmer">
                <div class="c-dimmer-content">
                    <p>A game is currently active</p>

                    <Button @click="gameStore.isDimmed = false"> Keep Using App </Button>
                </div>
            </div>
        </transition>

        <component :is="layout">
            <Suspense>
                <RouterView v-slot="{ Component }">
                    <transition name="fade-page" mode="out-in">
                        <component :is="Component" :key="route.fullPath" />
                    </transition>
                </RouterView>
            </Suspense>
        </component>
    </div>
</template>

<style lang="scss">
@use "./styles/main.scss";
</style>
