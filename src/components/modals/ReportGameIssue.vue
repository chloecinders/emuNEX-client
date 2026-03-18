<script lang="ts" setup>
import { ref } from 'vue';
import { http } from '../../utils/http';
import Modal from '../ui/Modal.vue';
import Button from '../ui/Button.vue';

const showModal = defineModel<boolean>("showModal");
const props = defineProps<{
    gameId: string;
}>();

const reportDescription = ref("");
const reportType = ref("incorrect_metadata");
const isSubmittingReport = ref(false);
const reportNotice = ref<{ type: "success" | "error"; message: string } | null>(null);

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
        <div style="display: flex; flex-direction: column; gap: 1rem; padding: 0.5rem 0;">
            <div v-if="reportNotice" class="c-report-notice" :class="`c-report-notice--${reportNotice.type}`">
                <Text variant="body" size="sm">{{ reportNotice.message }}</Text>
            </div>
            <div>
                <label
                    style="display: block; font-size: 0.85rem; font-weight: 700; color: var(--color-text-muted); margin-bottom: 0.5rem;">Issue
                    Type</label>
                <select v-model="reportType"
                    style="width: 100%; padding: 0.75rem; border-radius: var(--radius-sm); border: 1px solid var(--color-border); background: var(--color-surface); color: var(--color-text);">
                    <option value="incorrect_metadata">Incorrect Metadata</option>
                    <option value="bad_dump">Bad ROM Dump</option>
                    <option value="wrong_cover">Wrong Cover Image</option>
                    <option value="other">Other</option>
                </select>
            </div>
            <div>
                <label
                    style="display: block; font-size: 0.85rem; font-weight: 700; color: var(--color-text-muted); margin-bottom: 0.5rem;">Description</label>
                <textarea v-model="reportDescription" rows="4"
                    style="width: 100%; padding: 0.75rem; border-radius: var(--radius-sm); border: 1px solid var(--color-border); background: var(--color-surface); color: var(--color-text); font-family: inherit; resize: vertical;"
                    placeholder="Please describe the issue in detail..."></textarea>
            </div>
            <div class="c-modal-form__actions" style="margin-top: 0.5rem;">
                <Button class="c-button" color="primary" @click="submitReport"
                    :disabled="isSubmittingReport || !reportDescription.trim()">
                    {{ isSubmittingReport ? "Submitting..." : "Submit" }}
                </Button>
            </div>
        </div>
    </Modal>
</template>

<style lang="scss" scoped>
.c-report-notice {
    padding: 0.5rem;
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    box-shadow: var(--shadow-subtle);
}

.c-report-notice--success {
    background: var(--color-success);
    color: var(--color-text-on-success);
}

.c-report-notice--error {
    background: var(--color-error);
    color: var(--color-text-on-error);
}

.c-button {
    width: 100%;
}
</style>
