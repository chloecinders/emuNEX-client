<script setup lang="ts">
import Heading from "../components/ui/Heading.vue";
import Text from "../components/ui/Text.vue";
</script>

<template>
    <div class="c-about-view">
        <Heading>emuNEX</Heading>

        <Text
            >emuNEX is brought to you by @chloecinders and all
            <a href="https://github.com/chloecinders/emuNEX-client/graphs/contributors">contributors</a>. Without any of
            these people this project
        </Text>
    </div>
</template>

<style lang="scss" scoped>
.c-about-view {
    padding: var(--spacing-md) var(--spacing-lg) var(--spacing-xl) var(--spacing-lg);
}
</style>
