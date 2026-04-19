<script setup lang="ts">
import { Activity, Clock, Copy, FileOutput, FileText, Link, Search, Trash2 } from "lucide-vue-next";
import { computed, onMounted, onUnmounted, ref } from "vue";
import Heading from "../components/ui/Heading.vue";
import { useDevStore, type RequestLog } from "../stores/DevStore";

const devStore = useDevStore();
const searchQuery = ref("");
const selectedRequest = ref<RequestLog | null>(null);
const activeTab = ref<"request" | "response" | "preview">("response");

const isPreviewableHtml = computed(() => {
    const res = selectedRequest.value?.response;
    return typeof res === "string" && (/<html/i.test(res) || /<body/i.test(res) || /<!DOCTYPE/i.test(res));
});

const leftPaneWidth = ref(450);
const isDragging = ref(false);

const startDrag = () => {
    isDragging.value = true;
    document.body.style.cursor = "col-resize";
    document.addEventListener("mousemove", onDrag);
    document.addEventListener("mouseup", stopDrag);
};

const onDrag = (e: MouseEvent) => {
    if (!isDragging.value) return;
    leftPaneWidth.value = Math.max(200, Math.min(window.innerWidth - 300, e.clientX));
};

const stopDrag = () => {
    isDragging.value = false;
    document.body.style.cursor = "";
    document.removeEventListener("mousemove", onDrag);
    document.removeEventListener("mouseup", stopDrag);
};

onUnmounted(() => {
    document.removeEventListener("mousemove", onDrag);
    document.removeEventListener("mouseup", stopDrag);
});

const formatTime = (ts: number) => {
    return new Date(ts).toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
        hour12: false,
    });
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
    const map: Record<string, string> = {
        GET: "method--get",
        POST: "method--post",
        PUT: "method--put",
        DELETE: "method--delete",
    };
    return map[method.toUpperCase()] ?? "method--default";
};

const statusClass = (status: number) => {
    if (!status) return "status--error";
    if (status >= 200 && status < 300) return "status--success";
    if (status >= 400 && status < 500) return "status--warning";
    if (status >= 500) return "status--error";
    return "status--default";
};

const formatJson = (data: any) => {
    if (data === null || data === undefined) return String(data);
    if (typeof data === "string") return data;

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

const ctxMenu = ref({ show: false, x: 0, y: 0 });
const ctxRequest = ref<RequestLog | null>(null);

const openContextMenu = (e: MouseEvent, req: RequestLog) => {
    e.preventDefault();
    ctxRequest.value = req;
    ctxMenu.value = { show: true, x: e.clientX, y: e.clientY };
    setTimeout(() => {
        document.addEventListener("click", closeContextMenu, { once: true });
    }, 0);
};

const closeContextMenu = () => {
    ctxMenu.value.show = false;
};

const copyToClipboard = async (text: string) => {
    try {
        await navigator.clipboard.writeText(text);
    } catch {}
    closeContextMenu();
};

const copyPath = () => {
    if (!ctxRequest.value) return;
    copyToClipboard(pathOnly(ctxRequest.value.url));
};

const copyFullUrl = () => {
    if (!ctxRequest.value) return;
    copyToClipboard(ctxRequest.value.url);
};

const copyRequestBody = () => {
    if (!ctxRequest.value) return;
    copyToClipboard(formatJson(ctxRequest.value.body));
};

const copyResponseBody = () => {
    if (!ctxRequest.value) return;
    copyToClipboard(formatJson(ctxRequest.value.response));
};

const handleScroll = () => closeContextMenu();

onMounted(() => {
    window.addEventListener("scroll", handleScroll, true);
});

onUnmounted(() => {
    window.removeEventListener("scroll", handleScroll, true);
});
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

        <div class="c-rv__body" :class="{ 'c-rv__body--dragging': isDragging }">
            <template v-if="filteredRequests.length > 0">
                <div
                    class="c-rv__list-pane"
                    :style="selectedRequest ? { flex: `0 1 ${leftPaneWidth}px`, width: 'auto' } : {}"
                >
                    <div
                        v-for="req in filteredRequests"
                        :key="req.id"
                        class="c-rv__item"
                        :class="{
                            'c-rv__item--active': selectedRequest?.id === req.id,
                            'c-rv__item--error': !req.success,
                        }"
                        @click="selectRequest(req)"
                        @contextmenu="openContextMenu($event, req)"
                    >
                        <div class="c-rv__item-row">
                            <span class="c-rv__status" :class="statusClass(req.status)">{{ req.status || "ERR" }}</span>
                            <span class="c-rv__method" :class="methodClass(req.method)">{{ req.method }}</span>
                            <span class="c-rv__path">{{ pathOnly(req.url) }}</span>
                            <span class="c-rv__duration">
                                <Clock :size="10" />
                                {{ req.duration }}ms
                            </span>
                            <span class="c-rv__time">{{ formatTime(req.timestamp) }}</span>
                        </div>
                    </div>
                </div>

                <div
                    class="c-rv__divider"
                    :class="{ 'c-rv__divider--dragging': isDragging }"
                    @mousedown="startDrag"
                    v-if="selectedRequest"
                ></div>

                <div class="c-rv__detail-pane" v-if="selectedRequest">
                    <div class="c-rv__tabs">
                        <button
                            class="c-rv__tab"
                            :class="{ 'c-rv__tab--active': activeTab === 'request' }"
                            @click="activeTab = 'request'"
                            >Request</button
                        >
                        <button
                            class="c-rv__tab"
                            :class="{ 'c-rv__tab--active': activeTab === 'response' }"
                            @click="activeTab = 'response'"
                            >Response</button
                        >
                        <button
                            class="c-rv__tab"
                            :class="{ 'c-rv__tab--active': activeTab === 'preview' }"
                            @click="activeTab = 'preview'"
                            >Preview</button
                        >
                    </div>

                    <div class="c-rv__tab-content">
                        <div v-if="activeTab === 'request'" class="c-rv__tab-panel">
                            <div class="c-rv__detail-label">Request Body</div>
                            <pre class="c-rv__code" v-if="selectedRequest.body">{{
                                formatJson(selectedRequest.body)
                            }}</pre>
                            <div class="c-rv__empty" v-else>No request body</div>
                        </div>

                        <div v-if="activeTab === 'response'" class="c-rv__tab-panel">
                            <div class="c-rv__detail-label">Response Body</div>
                            <pre class="c-rv__code">{{ formatJson(selectedRequest.response) }}</pre>
                        </div>

                        <div v-if="activeTab === 'preview'" class="c-rv__tab-panel c-rv__tab-panel--preview">
                            <div class="c-rv__preview-wrapper" v-if="isPreviewableHtml">
                                <iframe
                                    :srcdoc="selectedRequest!.response"
                                    class="c-rv__preview-iframe"
                                    sandbox=""
                                ></iframe>
                            </div>
                            <div class="c-rv__empty" v-else>Preview not available for this response data.</div>
                        </div>
                    </div>
                </div>
            </template>

            <div class="c-rv__empty-state" v-else>
                <Activity :size="32" class="c-rv__empty-icon" />
                <p>No requests captured yet.</p>
                <p class="c-rv__empty-sub">Requests will appear here as you use the app.</p>
            </div>
        </div>
    </div>

    <div
        v-if="ctxMenu.show"
        class="c-rv-ctx"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
        @contextmenu.prevent
    >
        <button class="c-rv-ctx__item" @click="copyPath">
            <Link :size="14" />
            <span>Copy Path</span>
        </button>
        <button class="c-rv-ctx__item" @click="copyFullUrl">
            <Copy :size="14" />
            <span>Copy Full URL</span>
        </button>
        <div class="c-rv-ctx__separator"></div>
        <button class="c-rv-ctx__item" @click="copyRequestBody" :disabled="!ctxRequest?.body">
            <FileText :size="14" />
            <span>Copy Request Body</span>
        </button>
        <button class="c-rv-ctx__item" @click="copyResponseBody">
            <FileOutput :size="14" />
            <span>Copy Response Body</span>
        </button>
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
        display: flex;
        overflow: hidden;
        min-height: 0;

        &--dragging {
            user-select: none;
        }
    }

    &__list-pane {
        flex: 1;
        overflow-y: auto;
        padding: 8px;
        display: flex;
        flex-direction: column;
        gap: 4px;
        min-width: 0;
        min-height: 0;
    }

    &__divider {
        width: 1px;
        background: var(--color-border);
        cursor: col-resize;
        flex-shrink: 0;
        position: relative;
        z-index: 10;
        transition: background 0.15s ease;

        &::before {
            content: "";
            position: absolute;
            left: -3px;
            right: -3px;
            top: 0;
            bottom: 0;
            cursor: col-resize;
            background: transparent;
        }

        &:hover,
        &--dragging {
            background: var(--color-primary);
        }
    }

    &__item {
        background: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        cursor: pointer;
        transition: border-color 0.15s ease;
        overflow: hidden;
        flex-shrink: 0;

        &:hover {
            border-color: var(--color-primary);
        }

        &--active {
            border-color: var(--color-primary);
        }
    }

    &__item-row {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 8px 12px;
        font-size: 0.8rem;
    }

    &__status {
        font-size: 0.65rem;
        font-weight: 900;
        padding: 2px 6px;
        border-radius: var(--radius-sm);
        flex-shrink: 0;

        &.status--success {
            color: #22c55e;
            background: rgba(34, 197, 94, 0.1);
        }
        &.status--warning {
            color: #fb923c;
            background: rgba(251, 146, 60, 0.1);
        }
        &.status--error {
            color: #e60012;
            background: rgba(230, 0, 18, 0.1);
        }
        &.status--default {
            color: var(--color-text-muted);
            background: var(--color-surface-variant);
        }
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

    &__detail-pane {
        flex: 1;
        display: flex;
        flex-direction: column;
        background: var(--color-surface);
        overflow: hidden;
        min-width: 250px;
        min-height: 0;
    }

    &__tabs {
        display: flex;
        border-bottom: 1px solid var(--color-border);
    }

    &__tab {
        padding: 10px 16px;
        font-size: 0.75rem;
        font-weight: 600;
        color: var(--color-text-muted);
        background: transparent;
        border: none;
        border-bottom: 2px solid transparent;
        cursor: pointer;
        transition: all 0.15s ease;

        &:hover {
            color: var(--color-text);
        }

        &--active {
            color: var(--color-primary);
            border-bottom-color: var(--color-primary);
        }
    }

    &__tab-content {
        flex: 1;
        overflow-y: auto;
        padding: 16px;
    }

    &__tab-panel {
        display: flex;
        flex-direction: column;
        gap: 12px;
        height: 100%;

        &--preview {
            gap: 0;
            padding-bottom: 0;
        }
    }

    &__preview-wrapper {
        display: flex;
        flex: 1;
        width: 100%;
        min-height: 400px;
    }

    &__preview-iframe {
        width: 100%;
        height: 100%;
        border: 1px solid var(--color-border);
        border-radius: var(--radius-md);
        background: #ffffff;
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

.c-rv-ctx {
    position: fixed;
    z-index: 999999;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-lg);
    padding: 4px 0;
    min-width: 180px;

    &__item {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        text-align: left;
        background: transparent;
        border: none;
        padding: 7px 12px;
        color: var(--color-text);
        cursor: pointer;
        font-family: inherit;
        font-size: 0.8rem;

        &:hover {
            background: var(--color-surface-variant);
        }

        &:disabled {
            opacity: 0.35;
            pointer-events: none;
        }
    }

    &__separator {
        height: 1px;
        background: var(--color-border);
        margin: 4px 0;
    }
}
</style>
