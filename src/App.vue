<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import DefaultLayout from "./layouts/DefaultLayout.vue";
import TitleBar from "./components/ui/TitleBar.vue";

const route = useRoute();

const layout = computed(() => {
    return route.meta.layout || DefaultLayout;
});
</script>

<template>
    <TitleBar />
    
    <div class="app-main">
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
</style>
