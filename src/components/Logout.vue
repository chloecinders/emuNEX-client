<template>
    <Button @click="onClick">Logout</Button>
</template>

<script lang="ts" setup>
import { getGlobalStore, getDomainStore } from "../lib/store";
import { useAuthStore } from "../stores/AuthStore";
import Button from "./ui/Button.vue";

const authStore = useAuthStore();

const onClick = async () => {
    const currentDomain = authStore.domain;
    
    if (currentDomain) {
        const domainStore = await getDomainStore(currentDomain);
        await domainStore.set("token", null);
        await domainStore.save();
    }

    const globalStore = await getGlobalStore();

    await globalStore.set("domain", null);
    await globalStore.save();

    authStore.token = null;
    authStore.domain = null;

    // Use href to force a full reload and clear all injected state
    window.location.href = "/login";
};
</script>
