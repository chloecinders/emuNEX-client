<template>
    <Button @click="onClick">Logout</Button>
</template>

<script lang="ts" setup>
import { load } from "@tauri-apps/plugin-store";
import { provide } from "vue";
import { router } from "../router";
import { useAuthStore } from "../stores/AuthStore";
import Button from "./ui/Button.vue";

const authStore = useAuthStore();

const onClick = async () => {
    const store = await load("store.json", {
        autoSave: false,
        defaults: {},
    });

    provide("domain", null);
    provide("token", null);

    await store.set("domain", null);
    await store.set("token", null);
    await store.save();

    authStore.token = null;
    authStore.domain = null;

    router.push("/login");
};
</script>
