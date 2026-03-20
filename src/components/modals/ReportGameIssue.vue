<script lang="ts" setup>
import { ref } from 'vue';
import { http } from '../../utils/http';
import Modal from '../ui/Modal.vue';
import Button from '../ui/Button.vue';
import Select from '../ui/Select.vue';
import Text from '../ui/Text.vue';

const showModal = defineModel<boolean>("showModal");
const props = defineProps<{
    gameId: string;
}>();

const reportDescription = ref("");
const reportType = ref("incorrect_metadata");
const isSubmittingReport = ref(false);
const reportNotice = ref<{ type: "success" | "error"; message: string } | null>(null);

const reportOptions = [
    { name: "Incorrect Metadata", value: "incorrect_metadata" },
    { name: "Bad ROM Dump", value: "bad_dump" },
    { name: "Wrong Cover Image", value: "wrong_cover" },
    { name: "Other", value: "other" },
];

const submitReport = async () => {
    if (!reportDescription.value.trim()) return;

    try {
        isSubmittingReport.value = true;
        const res = await http.post(`/reports`, {
            rom_id: props.gameId,
            report_type: reportType.value,
            description: reportDescription.value,
        });

        if (res.success) {
            reportDescription.value = "";
            reportNotice.value = { type: "success", message: "Report submitted. Thanks!" };

            window.setTimeout(() => {
                showModal.value = false;
                reportNotice.value = null;
            }, 1200);
        } else {
            reportNotice.value = { type: "error", message: "Failed to submit report." };
        }
    } catch (e) {
        reportNotice.value = { type: "error", message: "Network error. Please try again." };
    } finally {
        isSubmittingReport.value = false;
    }
};
</script>

<template>
    <Modal :show="showModal || false" title="Report Game Issue" @close="showModal = false">
        <div class="c-report-form">
            <div v-if="reportNotice" class="c-report-notice" :class="`c-report-notice--${reportNotice.type}`">
                <Text variant="body" size="sm">{{ reportNotice.message }}</Text>
            </div>

            <Select
                v-model="reportType"
                label="Issue Type"
                :options="reportOptions"
            />

            <div class="c-report-form__field">
                <label class="c-report-form__label">Description</label>
                <textarea
                    v-model="reportDescription"
                    rows="4"
                    class="c-report-form__textarea"
                    placeholder="Please describe the issue in detail..."
                ></textarea>
            </div>

            <div class="c-report-form__actions">
                <Button
                    class="c-button"
                    color="primary"
                    @click="submitReport"
                    :disabled="isSubmittingReport || !reportDescription.trim()"
                >
                    {{ isSubmittingReport ? "Submitting..." : "Submit" }}
                </Button>
            </div>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-report-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-lg);
    padding: var(--spacing-xs) 0;

    &__field {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs);
    }

    &__label {
        display: block;
        font-size: 0.8rem;
        font-weight: 800;
        color: var(--color-text-muted);
        margin-left: var(--spacing-xs);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    &__textarea {
        width: 100%;
        padding: var(--spacing-md);
        border-radius: var(--radius-md);
        border: 2px solid var(--color-border);
        background: var(--color-surface);
        color: var(--color-text);
        font-family: inherit;
        font-size: 1rem;
        font-weight: 600;
        resize: vertical;
        transition: border-color 0.2s ease;

        &:focus {
            outline: none;
            border-color: var(--color-primary);
        }

        &::placeholder {
            opacity: 0.5;
        }
    }

    &__actions {
        margin-top: var(--spacing-sm);

        .c-button {
            width: 100%;
        }
    }
}

.c-report-notice {
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);

    &--success {
        border-color: color-mix(in srgb, var(--color-success, #22c55e) 45%, var(--color-border));
        background: color-mix(in srgb, var(--color-success, #22c55e) 12%, transparent);
    }

    &--error {
        border-color: color-mix(in srgb, var(--color-danger) 45%, var(--color-border));
        background: color-mix(in srgb, var(--color-danger) 12%, transparent);
    }
}
</style>
