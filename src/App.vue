<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import DefaultLayout from "./layouts/DefaultLayout.vue";
import TitleBar from "./components/ui/TitleBar.vue";
import Button from "./components/ui/Button.vue";
import { useGameStore } from "./stores/GameStore";

const route = useRoute();
const gameStore = useGameStore();

const layout = computed(() => {
    return route.meta.layout || DefaultLayout;
});
</script>

<template>
    <TitleBar />
    
    <div class="c-app-main">
        <transition name="dim">
            <div v-if="gameStore.isDimmed" class="c-app-dimmer">
                <div class="c-dimmer-content">
                    <p>A game is currently active</p>

                    <Button @click="gameStore.isDimmed = false">
                        Keep Using App
                    </Button>
                </div>
            </div>
        </transition>

        <component :is="layout">
            <Suspense>
                <RouterView />
            </Suspense>
        </component>
    </div>
</template>

<style lang="scss">
@use "./styles/main.scss";
</style>
