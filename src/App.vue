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
    
    <div class="app-main">
        <transition name="dim">
            <div v-if="gameStore.isDimmed" class="app-dimmer">
                <div class="dimmer-content">
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

<style>
* {
    box-sizing: border-box;
}

:root {
    --color-primary: #6B5CB1;
    --color-primary-light: #8BA0E0;
    --color-primary-dark: #573795;
    --color-secondary: #CEC1E3;
    --color-accent: #ff7e12;
    --color-background: #f0f0f0;
    --color-surface: #ffffff;
    --color-surface-variant: #f8f8fc;
    --color-text: #2d2d3a;
    --color-text-muted: #6e6e80;
    --color-text-on-primary: #ffffff;
    --color-border: #e2e2ec;
    --color-border-hover: #d0d0e0;

    --spacing-xxs: 2px;
    --spacing-xs: 4px;
    --spacing-sm: 8px;
    --spacing-md: 16px;
    --spacing-lg: 24px;
    --spacing-xl: 32px;
    --spacing-xxl: 48px;

    --radius-sm: 8px;
    --radius-md: 12px;
    --radius-lg: 20px;
    --radius-full: 9999px;

    --shadow-subtle: 0 2px 8px rgba(107, 92, 177, 0.08);
    --shadow-md: 0 4px 20px rgba(107, 92, 177, 0.12);
    --shadow-lg: 0 8px 32px rgba(10, 10, 20, 0.15);

    --glass-bg: rgba(255, 255, 255, 0.7);
    --glass-border: rgba(255, 255, 255, 0.3);

    --titlebar-height: 32px;
}

.app-main {
    margin-top: var(--titlebar-height);
    height: calc(100vh - var(--titlebar-height));
    position: relative;
    overflow: hidden;
}

body {
    margin: 0;
    padding: 0;
    font-family: 'Inter', sans-serif;
    color: var(--color-text);
    background-color: var(--color-background);
    background-image: radial-gradient(#dcdcdc var(--spacing-xxs), transparent var(--spacing-xxs));
    background-size: 20px 20px;
    background-attachment: fixed;
    overflow: hidden;
    line-height: 1.5;
    -webkit-font-smoothing: antialiased;
}

h1, h2, h3, h4 {
    margin-top: 0;
    font-weight: 800;
}

.app-dimmer {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
}

.dimmer-content {
    background: var(--color-surface);
    padding: var(--spacing-xl);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    text-align: center;
    border: 1px solid var(--color-border);
}

.dimmer-content p {
    margin: 0 0 var(--spacing-lg) 0;
    font-weight: 800;
    color: var(--color-text);
    font-size: 1.2rem;
}

.dim-enter-active,
.dim-leave-active {
    transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.dim-enter-from,
.dim-leave-to {
    opacity: 0;
    backdrop-filter: blur(0px);
}
</style>
