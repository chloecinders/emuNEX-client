<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import DefaultLayout from "./layouts/DefaultLayout.vue";

const route = useRoute();

const layout = computed(() => {
    return route.meta.layout || DefaultLayout;
});
</script>

<template>
    <component :is="layout">
        <Suspense>
            <RouterView />
        </Suspense>
    </component>
</template>

<style>
* {
    box-sizing: border-box;
}
:root {
    --3ds-bg: #f0f0f0;
    --3ds-red: #e60012;
    --3ds-blue: #0089cf;
    --3ds-border: #ccc;
    --3ds-text: #444;
}
body {
    background-color: var(--3ds-bg);
    background-image: radial-gradient(#dcdcdc 2px, transparent 2px);
    background-size: 20px 20px;
    margin: 0;
    padding: 0;
    font-family: sans-serif;
}
</style>
