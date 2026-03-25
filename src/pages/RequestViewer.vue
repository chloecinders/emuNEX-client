<script setup lang="ts">
import { computed, ref } from "vue";
import { Activity, ChevronDown, ChevronUp, Clock, Search, Trash2 } from "lucide-vue-next";
import Heading from "../components/ui/Heading.vue";
import { useDevStore, type RequestLog } from "../stores/DevStore";

const devStore = useDevStore();
const searchQuery = ref("");
const selectedRequest = ref<RequestLog | null>(null);

const formatTime = (ts: number) => {
    return new Date(ts).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: false });
};

const filteredRequests = computed(() => {
    const q = searchQuery.value.toLowerCase();
    if (!q) return devStore.requests;
    return devStore.requests.filter((r) => r.url.toLowerCase().includes(q) || r.method.toLowerCase().includes(q));
});

const selectRequest = (req: RequestLog) => {
    selectedRequest.value = selectedRequest.value?.id === req.id ? null : req;
};

const methodClass = (method: string) => {
    const map: Record<string, string> = { GET: "method--get", POST: "method--post", PUT: "method--put", DELETE: "method--delete" };
    return map[method.toUpperCase()] ?? "method--default";
};

const formatJson = (data: any) => {
    try {
        return JSON.stringify(data, null, 2);
    } catch {
        return String(data);
    }
};

const pathOnly = (url: string) => {
    try {
        return new URL(url).pathname + new URL(url).search;
    } catch {
        return url;
    }
};
</script>

<template>
    <div class="c-rv">
        <div class="c-rv__header">
            <div class="c-rv__header-left">
                <Activity :size="18" class="c-rv__icon" />
                <Heading level="3">Request Viewer</Heading>
                <span class="c-rv__count">{{ devStore.requests.length }}</span>
            </div>
            <div class="c-rv__header-right">
                <div class="c-rv__search-wrap">
                    <Search :size="14" class="c-rv__search-icon" />
                    <input v-model="searchQuery" class="c-rv__search" placeholder="Filter requests..." />
                </div>
                <button class="c-rv__clear" @click="devStore.clearRequests()" title="Clear all">
                    <Trash2 :size="14" />
                    <span>Clear</span>
                </button>
            </div>
        </div>

        <div class="c-rv__body">
            <div class="c-rv__list" v-if="filteredRequests.length > 0">
                <div
                    v-for="req in filteredRequests"
                    :key="req.id"
                    class="c-rv__item"
                    :class="{ 'c-rv__item--active': selectedRequest?.id === req.id, 'c-rv__item--error': !req.success }"
                    @click="selectRequest(req)"
                >
                    <div class="c-rv__item-row">
                        <span class="c-rv__method" :class="methodClass(req.method)">{{ req.method }}</span>
                        <span class="c-rv__path">{{ pathOnly(req.url) }}</span>
                        <span class="c-rv__duration">
                            <Clock :size="10" />
                            {{ req.duration }}ms
                        </span>
                        <span class="c-rv__time">{{ formatTime(req.timestamp) }}</span>
                        <ChevronDown v-if="selectedRequest?.id !== req.id" :size="14" class="c-rv__chevron" />
                        <ChevronUp v-else :size="14" class="c-rv__chevron" />
                    </div>

                    <Transition name="expand">
                        <div v-if="selectedRequest?.id === req.id" class="c-rv__detail" @click.stop>
                            <div class="c-rv__detail-grid">
                                <div class="c-rv__detail-panel">
                                    <div class="c-rv__detail-label">Request Body</div>
                                    <pre class="c-rv__code" v-if="req.body">{{ formatJson(req.body) }}</pre>
                                    <div class="c-rv__empty" v-else>No request body</div>
                                </div>
                                <div class="c-rv__detail-panel">
                                    <div class="c-rv__detail-label">Response</div>
                                    <pre class="c-rv__code">{{ formatJson(req.response) }}</pre>
                                </div>
                            </div>
                        </div>
                    </Transition>
                </div>
            </div>

            <div class="c-rv__empty-state" v-else>
                <Activity :size="32" class="c-rv__empty-icon" />
                <p>No requests captured yet.</p>
                <p class="c-rv__empty-sub">Requests will appear here as you use the app.</p>
            </div>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.c-rv {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg);
    font-family: inherit;

    &__header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 12px 16px;
        border-bottom: 1px solid var(--color-border);
        background: var(--color-surface);
        gap: 12px;
        flex-shrink: 0;
    }

    &__header-left {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    &__header-right {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    &__icon {
        color: var(--color-primary);
    }

    &__count {
        background: var(--color-primary);
        color: white;
        font-size: 0.65rem;
        font-weight: 800;
        border-radius: 999px;
        padding: 2px 7px;
    }

    &__search-wrap {
        position: relative;
        display: flex;
        align-items: center;
    }

    &__search-icon {
        position: absolute;
        left: 10px;
        color: var(--color-text-muted);
        pointer-events: none;
    }

    &__search {
        background: var(--color-surface-variant);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        padding: 6px 10px 6px 30px;
        font-size: 0.8rem;
        color: var(--color-text);
        width: 220px;
        outline: none;

        &::placeholder {
            color: var(--color-text-muted);
        }

        &:focus {
            border-color: var(--color-primary);
        }
    }

    &__clear {
        display: flex;
        align-items: center;
        gap: 5px;
        background: var(--color-surface-variant);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        color: var(--color-text-muted);
        font-size: 0.75rem;
        cursor: pointer;
        padding: 6px 10px;
        transition: all 0.15s ease;

        &:hover {
            border-color: #e60012;
            color: #e60012;
        }
    }

    &__body {
        flex: 1;
        overflow-y: auto;
        padding: 8px;
    }

    &__list {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    &__item {
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        cursor: pointer;
        transition: border-color 0.15s ease;
        overflow: hidden;

        &:hover {
            border-color: var(--color-primary);
        }

        &--active {
            border-color: var(--color-primary);
        }

        &--error {
            border-left: 3px solid #e60012;
        }
    }

    &__item-row {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 12px;
        font-size: 0.8rem;
    }

    &__method {
        font-size: 0.65rem;
        font-weight: 900;
        padding: 2px 7px;
        border-radius: var(--radius-sm);
        text-transform: uppercase;
        letter-spacing: 0.5px;
        flex-shrink: 0;

        &.method--get {
            background: rgba(34, 197, 94, 0.15);
            color: #22c55e;
        }
        &.method--post {
            background: rgba(59, 130, 246, 0.15);
            color: #3b82f6;
        }
        &.method--put {
            background: rgba(251, 146, 60, 0.15);
            color: #fb923c;
        }
        &.method--delete {
            background: rgba(230, 0, 18, 0.15);
            color: #e60012;
        }
        &.method--default {
            background: var(--color-surface-variant);
            color: var(--color-text-muted);
        }
    }

    &__path {
        flex: 1;
        font-family: "Courier New", monospace;
        font-size: 0.78rem;
        color: var(--color-text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    &__duration {
        display: flex;
        align-items: center;
        gap: 3px;
        font-size: 0.7rem;
        color: var(--color-text-muted);
        flex-shrink: 0;
    }

    &__time {
        font-size: 0.7rem;
        color: var(--color-text-muted);
        flex-shrink: 0;
        font-variant-numeric: tabular-nums;
    }

    &__chevron {
        color: var(--color-text-muted);
        flex-shrink: 0;
    }

    &__detail {
        border-top: 1px solid var(--color-border);
        padding: 12px;
        background: var(--color-bg);
    }

    &__detail-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 12px;

        @media (max-width: 600px) {
            grid-template-columns: 1fr;
        }
    }

    &__detail-panel {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    &__detail-label {
        font-size: 0.65rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.8px;
        color: var(--color-text-muted);
    }

    &__code {
        background: var(--color-surface-variant);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        padding: 10px;
        font-family: "Courier New", monospace;
        font-size: 0.75rem;
        color: var(--color-text);
        overflow: auto;
        max-height: 300px;
        margin: 0;
        white-space: pre-wrap;
        word-break: break-all;
    }

    &__empty {
        font-size: 0.75rem;
        color: var(--color-text-muted);
        font-style: italic;
        padding: 8px;
    }

    &__empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        min-height: 200px;
        gap: 8px;
        color: var(--color-text-muted);
        text-align: center;

        p {
            margin: 0;
            font-size: 0.95rem;
        }
    }

    &__empty-icon {
        opacity: 0.3;
    }

    &__empty-sub {
        font-size: 0.8rem !important;
        opacity: 0.7;
    }
}

.expand-enter-active,
.expand-leave-active {
    transition: opacity 0.15s ease;
}
.expand-enter-from,
.expand-leave-to {
    opacity: 0;
}
</style>
