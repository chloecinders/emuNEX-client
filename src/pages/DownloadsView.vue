<script setup lang="ts">
import { AlertCircle, CheckCircle, Clock, Download, X, Zap } from "lucide-vue-next";
import { computed } from "vue";
import Heading from "../components/ui/Heading.vue";
import { useDownloadStore, type DownloadItem } from "../stores/DownloadStore";

const store = useDownloadStore();

const active = computed(() => store.items.filter((i) => i.status === "downloading"));
const queued = computed(() => store.items.filter((i) => i.status === "queued"));
const history = computed(() =>
    store.items
        .filter((i) => i.status === "done" || i.status === "error" || i.status === "cancelled")
        .slice()
        .reverse(),
);

function formatBytes(bytes: number): string {
    if (bytes <= 0) return "0 B";
    const units = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[Math.min(i, 3)]}`;
}

function formatSpeed(bps: number): string {
    if (bps <= 0) return "-";
    return `${formatBytes(bps)}/s`;
}

function formatEta(item: DownloadItem): string {
    if (item.speed_bps <= 0 || item.total_bytes <= 0) return "-";
    const remaining = item.total_bytes - item.downloaded_bytes;
    const secs = Math.ceil(remaining / item.speed_bps);
    if (secs < 60) return `${secs}s`;
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}m ${s}s`;
}

function progressPct(item: DownloadItem): number {
    if (!item.total_bytes) return 0;
    return Math.min(100, Math.round((item.downloaded_bytes / item.total_bytes) * 100));
}

function formatDuration(item: DownloadItem): string {
    if (!item.started_at || !item.completed_at) return "";
    const ms = item.completed_at - item.started_at;
    const s = Math.ceil(ms / 1000);
    if (s < 60) return `${s}s`;
    return `${Math.floor(s / 60)}m ${s % 60}s`;
}

function avgSpeed(item: DownloadItem): string {
    if (!item.started_at || !item.completed_at || !item.downloaded_bytes) return "";
    const ms = item.completed_at - item.started_at;
    if (ms <= 0) return "";
    const bps = Math.round((item.downloaded_bytes / ms) * 1000);
    return formatSpeed(bps);
}

function kindBadge(item: DownloadItem): string {
    return item.kind.kind === "rom" ? "ROM" : "EMULATOR";
}

function getInstallingLabel(item: DownloadItem): string {
    const pctStr = item.extraction_progress !== undefined ? ` ${item.extraction_progress}%` : "";
    if (item.kind.kind === "rom" && (item.kind as any).zipped) {
        return `Unpacking & Verifying...${pctStr}`;
    }
    return `Installing & Verifying...${pctStr}`;
}
</script>

<template>
    <div class="c-downloads">
        <div class="c-downloads__header">
            <div class="c-downloads__header-left">
                <Download class="c-downloads__header-icon" />
                <Heading :level="2" class="c-downloads__title">Downloads</Heading>
            </div>
        </div>

        <section v-if="active.length > 0" class="c-downloads__section">
            <div class="c-downloads__section-label">Downloading</div>

            <div class="c-downloads__list">
                <div v-for="item in active" :key="item.id" class="c-downloads__card c-downloads__card--active">
                    <div class="c-downloads__card-header">
                        <div class="c-downloads__card-info">
                            <span class="c-downloads__kind-badge">{{ kindBadge(item) }}</span>
                            <span class="c-downloads__card-name">{{ item.label }}</span>
                        </div>
                        <button class="c-downloads__cancel-btn" title="Cancel" @click="store.cancel(item.id)">
                            <X :size="14" />
                        </button>
                    </div>

                    <div class="c-downloads__progress-bar-wrap">
                        <div
                            class="c-downloads__progress-bar"
                            :style="{
                                width:
                                    progressPct(item) === 100 && item.extraction_progress !== undefined
                                        ? item.extraction_progress + '%'
                                        : progressPct(item) + '%',
                            }" />
                    </div>

                    <div class="c-downloads__stats">
                        <template v-if="progressPct(item) < 100">
                            <span class="c-downloads__stat">
                                <Zap :size="12" class="c-downloads__stat-icon" />
                                {{ formatSpeed(item.speed_bps) }}
                            </span>
                            <span class="c-downloads__stat">
                                <Clock :size="12" class="c-downloads__stat-icon" />
                                {{ formatEta(item) }} remaining
                            </span>
                        </template>
                        <template v-else>
                            <span class="c-downloads__stat c-downloads__stat--installing">
                                <Clock :size="12" class="c-downloads__stat-icon" />
                                {{ getInstallingLabel(item) }}
                            </span>
                        </template>
                        <span class="c-downloads__stat c-downloads__stat--right">
                            {{ formatBytes(item.downloaded_bytes) }}
                            {{ item.total_bytes ? `/ ${formatBytes(item.total_bytes)}` : "" }}
                            &nbsp;·&nbsp; {{ progressPct(item) }}%
                        </span>
                    </div>
                </div>
            </div>
        </section>

        <section v-if="queued.length > 0" class="c-downloads__section">
            <div class="c-downloads__section-label">Queued</div>
            <div class="c-downloads__list">
                <div v-for="item in queued" :key="item.id" class="c-downloads__card c-downloads__card--queued">
                    <div class="c-downloads__card-header">
                        <div class="c-downloads__card-info">
                            <span class="c-downloads__kind-badge c-downloads__kind-badge--muted">
                                {{ kindBadge(item) }}
                            </span>
                            <span class="c-downloads__card-name">{{ item.label }}</span>
                        </div>
                        <div class="c-downloads__card-right">
                            <span v-if="item.total_bytes" class="c-downloads__size-label">
                                {{ formatBytes(item.total_bytes) }}
                            </span>
                            <button
                                class="c-downloads__cancel-btn"
                                title="Remove from queue"
                                @click="store.cancel(item.id)">
                                <X :size="14" />
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </section>

        <section v-if="history.length > 0" class="c-downloads__section">
            <div class="c-downloads__section-label">History</div>
            <div class="c-downloads__list">
                <div
                    v-for="item in history"
                    :key="item.id"
                    class="c-downloads__card c-downloads__card--history"
                    :class="{
                        'c-downloads__card--done': item.status === 'done',
                        'c-downloads__card--error': item.status === 'error',
                        'c-downloads__card--cancelled': item.status === 'cancelled',
                    }">
                    <div class="c-downloads__card-header">
                        <div class="c-downloads__card-info">
                            <component
                                :is="item.status === 'done' ? CheckCircle : AlertCircle"
                                :size="15"
                                class="c-downloads__history-icon" />
                            <span class="c-downloads__kind-badge c-downloads__kind-badge--muted">
                                {{ kindBadge(item) }}
                            </span>
                            <span class="c-downloads__card-name">{{ item.label }}</span>
                        </div>
                        <div class="c-downloads__card-right">
                            <template v-if="item.status === 'done'">
                                <span class="c-downloads__history-stat">{{ avgSpeed(item) }}</span>
                                <span class="c-downloads__history-stat">{{ formatDuration(item) }}</span>
                                <span class="c-downloads__history-stat">{{ formatBytes(item.downloaded_bytes) }}</span>
                            </template>
                            <span v-else-if="item.status === 'error'" class="c-downloads__error-msg">
                                {{ item.error }}
                            </span>
                            <span v-else class="c-downloads__cancelled-label">Cancelled</span>

                            <button
                                class="c-downloads__cancel-btn"
                                title="Remove from history"
                                @click="store.remove(item.id)">
                                <X :size="14" />
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </section>

        <div v-if="active.length === 0 && queued.length === 0 && history.length === 0" class="c-downloads__empty">
            <div class="c-downloads__empty-icon-wrap">
                <Download class="c-downloads__empty-icon" />
            </div>
            <Heading :level="3" class="c-downloads__empty-title">No downloads yet</Heading>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-downloads {
    padding: var(--spacing-xl);
    max-width: 860px;
    margin: 0 auto;
    width: 100%;
    box-sizing: border-box;

    &__header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: var(--spacing-xl);
    }

    &__header-left {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
    }

    &__header-icon {
        width: 24px;
        height: 24px;
        color: var(--color-primary);
        stroke-width: 2.5px;
    }

    &__title {
        margin: 0;
    }

    &__section {
        margin-bottom: var(--spacing-xl);
    }

    &__section-label {
        font-size: 0.7rem;
        font-weight: 800;
        text-transform: uppercase;
        letter-spacing: 1px;
        color: var(--color-text-muted);
        margin-bottom: var(--spacing-sm);
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
    }

    &__pulse {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: var(--color-primary);
        animation: pulse 1.5s ease-in-out infinite;
        flex-shrink: 0;
    }

    &__list {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
    }

    &__card {
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        padding: var(--spacing-md) var(--spacing-lg);
        transition: border-color 0.2s;

        &--active {
            border-color: color-mix(in srgb, var(--color-primary) 40%, var(--color-border));
        }

        &--error {
            border-color: color-mix(in srgb, var(--color-danger) 40%, var(--color-border));
        }
    }

    &__card-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: var(--spacing-md);
        margin-bottom: var(--spacing-xs);

        .c-downloads__card--history & {
            margin-bottom: 0;
        }

        .c-downloads__card--queued & {
            margin-bottom: 0;
        }
    }

    &__card-info {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        min-width: 0;
        flex: 1;
    }

    &__card-name {
        font-weight: 600;
        font-size: 0.9rem;
        color: var(--color-text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    &__card-right {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
        flex-shrink: 0;
    }

    &__kind-badge {
        font-size: 0.6rem;
        font-weight: 900;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        background: color-mix(in srgb, var(--color-primary) 15%, transparent);
        color: var(--color-primary);
        padding: 2px 7px;
        border-radius: var(--radius-full);
        flex-shrink: 0;

        &--muted {
            background: var(--color-surface-variant);
            color: var(--color-text-muted);
        }
    }

    &__progress-bar-wrap {
        height: 4px;
        background: var(--color-border);
        border-radius: var(--radius-full);
        overflow: hidden;
        margin: var(--spacing-sm) 0 var(--spacing-xs);
    }

    &__progress-bar {
        height: 100%;
        background: var(--color-primary);
        border-radius: var(--radius-full);
        transition: width 0.3s ease;
    }

    &__stats {
        display: flex;
        align-items: center;
        gap: var(--spacing-lg);
        font-size: 0.78rem;
        font-weight: 600;
        color: var(--color-text-muted);
    }

    &__stat {
        display: flex;
        align-items: center;
        gap: 4px;

        &--right {
            margin-left: auto;
        }

        &--installing {
            color: var(--color-primary);
            font-weight: 700;
        }
    }

    &__stat-icon {
        opacity: 0.7;
    }

    &__cancel-btn {
        width: 26px;
        height: 26px;
        border-radius: var(--radius-full);
        border: 1px solid var(--color-border);
        background: transparent;
        color: var(--color-text-muted);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.15s;
        flex-shrink: 0;

        &:hover {
            border-color: var(--color-danger);
            color: var(--color-danger);
            background: color-mix(in srgb, var(--color-danger) 10%, transparent);
        }
    }

    &__size-label {
        font-size: 0.78rem;
        font-weight: 600;
        color: var(--color-text-muted);
    }

    &__history-icon {
        flex-shrink: 0;

        .c-downloads__card--done & {
            color: var(--color-success);
        }

        .c-downloads__card--error & {
            color: var(--color-danger);
        }

        .c-downloads__card--cancelled & {
            color: var(--color-text-muted);
        }
    }

    &__history-stat {
        font-size: 0.75rem;
        font-weight: 600;
        color: var(--color-text-muted);

        &:not(:last-child)::after {
            content: "·";
            margin-left: var(--spacing-md);
        }
    }

    &__error-msg {
        font-size: 0.75rem;
        font-weight: 600;
        color: var(--color-danger);
        max-width: 300px;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    &__cancelled-label {
        font-size: 0.75rem;
        font-weight: 700;
        color: var(--color-text-muted);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    &__empty {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: var(--spacing-xxl) var(--spacing-xl);
        gap: var(--spacing-md);
        text-align: center;
    }

    &__empty-icon-wrap {
        width: 72px;
        height: 72px;
        border-radius: var(--radius-full);
        background: var(--color-surface-variant);
        display: flex;
        align-items: center;
        justify-content: center;
        margin-bottom: var(--spacing-sm);
    }

    &__empty-icon {
        width: 32px;
        height: 32px;
        color: var(--color-primary);
    }

    &__empty-title {
        margin: 0;
    }
}
</style>
