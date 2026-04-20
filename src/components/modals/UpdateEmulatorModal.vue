<script setup lang="ts">
import { Download, HardDrive } from "lucide-vue-next";
import { type Emulator, type ServerEmulator } from "../../stores/EmulatorStore";
import Button from "../ui/Button.vue";
import Heading from "../ui/Heading.vue";
import Modal from "../ui/Modal.vue";
import Spinner from "../ui/Spinner.vue";
import Text from "../ui/Text.vue";

defineProps<{
    show: boolean;
    mode?: "update" | "refresh";
    localEmulator: Emulator | null;
    serverEmulator: ServerEmulator | null;
    loading?: boolean;
}>();

const emit = defineEmits<{
    (e: "close"): void;
    (e: "confirm", keepConfig: boolean): void;
}>();

const handleKeepConfig = () => {
    emit("confirm", true);
};

const handleApplyConfig = () => {
    emit("confirm", false);
};
</script>

<template>
    <Modal
        :show="show"
        :title="mode === 'refresh' ? 'Refresh Configuration' : 'Update Available'"
        width="640px"
        @close="emit('close')"
    >
        <div class="c-update-modal">
            <Text variant="muted" class="c-update-modal__description">
                <template v-if="mode === 'refresh'">
                    Compare your local settings for {{ localEmulator?.name }} with the latest server configuration.
                    Re-applying will overwrite your custom execution and path settings.
                </template>
                <template v-else>
                    A new version of {{ localEmulator?.name }} (v{{ serverEmulator?.version }}) is available. Choose
                    your update preference below.
                </template>
            </Text>

            <div v-if="loading" class="c-update-modal__loading">
                <Spinner />
                <Text>{{ mode === "refresh" ? "Refreshing configuration..." : "Applying update..." }}</Text>
            </div>

            <div v-else class="c-diff">
                <div class="c-diff__col">
                    <Heading :level="4" size="sm" class="c-diff__title">Local Config</Heading>
                    <div class="c-diff__group">
                        <Text variant="label" size="xs">Execution</Text>
                        <ul class="c-diff__list">
                            <li
                                ><span>Run Command</span> <code>{{ localEmulator?.run_command || "None" }}</code></li
                            >
                        </ul>
                    </div>
                    <div class="c-diff__group">
                        <Text variant="label" size="xs">Storage & Files</Text>
                        <ul class="c-diff__list">
                            <li
                                ><span>Save Path</span> <code>{{ localEmulator?.save_path || "None" }}</code></li
                            >
                            <li
                                ><span>Extensions</span>
                                <Text size="sm">{{ localEmulator?.save_extensions?.join(", ") || "None" }}</Text></li
                            >
                        </ul>
                    </div>
                    <div class="c-diff__group">
                        <Text variant="label" size="xs">Input & Mapping</Text>
                        <ul class="c-diff__list">
                            <li
                                ><span>Config File</span>
                                <code>{{ localEmulator?.input_config_file || "None" }}</code></li
                            >
                            <li
                                ><span>Mapper</span> <code>{{ localEmulator?.input_mapper || "None" }}</code></li
                            >
                        </ul>
                    </div>
                </div>

                <div class="c-diff__col c-diff__col--server">
                    <Heading :level="4" size="sm" class="c-diff__title c-diff__title--server">Server Config</Heading>
                    <div class="c-diff__group">
                        <Text variant="label" size="xs">Execution</Text>
                        <ul class="c-diff__list">
                            <li
                                ><span>Run Command</span> <code>{{ serverEmulator?.run_command || "None" }}</code></li
                            >
                        </ul>
                    </div>
                    <div class="c-diff__group">
                        <Text variant="label" size="xs">Storage & Files</Text>
                        <ul class="c-diff__list">
                            <li
                                ><span>Save Path</span> <code>{{ serverEmulator?.save_path || "None" }}</code></li
                            >
                            <li
                                ><span>Extensions</span>
                                <Text size="sm">{{ serverEmulator?.save_extensions?.join(", ") || "None" }}</Text></li
                            >
                        </ul>
                    </div>
                    <div class="c-diff__group">
                        <Text variant="label" size="xs">Input & Mapping</Text>
                        <ul class="c-diff__list">
                            <li
                                ><span>Config File</span>
                                <code>{{ serverEmulator?.input_config_file || "None" }}</code></li
                            >
                            <li
                                ><span>Mapper</span> <code>{{ serverEmulator?.input_mapper || "None" }}</code></li
                            >
                        </ul>
                    </div>
                </div>
            </div>

            <div class="c-update-modal__actions">
                <Button @click="handleKeepConfig" :disabled="loading" color="grey" full>
                    <HardDrive :size="18" /> {{ mode === "refresh" ? "Cancel" : "Keep Local Config" }}
                </Button>
                <Button @click="handleApplyConfig" :disabled="loading" color="primary" full>
                    <Download :size="18" /> {{ mode === "refresh" ? "Apply Server Config" : "Apply Server Config" }}
                </Button>
            </div>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-update-modal {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);

    &__description {
        line-height: 1.5;
    }

    &__loading {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-xl) 0;
    }

    &__actions {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-md);
        margin-top: var(--spacing-lg);
        padding-top: var(--spacing-xl);
        border-top: 1px solid var(--color-border);
    }
}

.c-diff {
    display: flex;
    gap: var(--spacing-md);

    &__col {
        flex: 1;
        background: var(--color-surface-variant);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        padding: var(--spacing-md);
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);

        &--server {
            border-color: color-mix(in srgb, var(--color-primary) 30%, transparent);
            background: color-mix(in srgb, var(--color-primary) 5%, transparent);
        }
    }

    &__group {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: var(--spacing-md);

        &:last-child {
            margin-bottom: 0;
        }

        :deep(.c-text--label) {
            color: var(--color-primary);
            opacity: 0.8;
            font-weight: 900;
            letter-spacing: 0.5px;
            text-transform: uppercase;
        }
    }

    &__title {
        border-bottom: 1px solid var(--color-border);
        padding-bottom: 8px;
        margin-bottom: var(--spacing-sm);

        &--server {
            color: var(--color-primary);
            border-bottom-color: color-mix(in srgb, var(--color-primary) 20%, transparent);
        }
    }

    &__list {
        list-style: none;
        padding: 0;
        margin: 0;
        display: flex;
        flex-direction: column;
        gap: 8px;

        li {
            font-size: 0.78rem;
            color: var(--color-text);
            display: flex;
            flex-direction: column;
            gap: 4px;

            span {
                font-weight: 700;
                color: var(--color-text-muted);
                font-size: 0.7rem;
            }

            code {
                font-family: var(--font-mono, "JetBrains Mono", monospace);
                background: rgba(0, 0, 0, 0.2);
                padding: 4px 8px;
                border-radius: var(--radius-sm);
                border: 1px solid rgba(255, 255, 255, 0.05);
                word-break: break-all;
                color: var(--color-text);
            }
        }
    }
}
</style>
