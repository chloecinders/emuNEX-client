import { emit, listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { ref, watch } from "vue";


export interface RequestLog {
    id: string;
    method: string;
    url: string;
    status: number;
    body: any;
    response: any;
    success: boolean;
    timestamp: number;
    duration: number;
}

export const useDevStore = defineStore("devStore", () => {
    const isDevMode = ref(localStorage.getItem("devMode") === "true");
    const requests = ref<RequestLog[]>([]);
    const isRequestsWindow = new URLSearchParams(window.location.search).has("dev");

    listen<RequestLog>("dev://request", (event) => {
        if (!requests.value.some((r) => r.id === event.payload.id)) {
            requests.value.unshift(event.payload);
            if (requests.value.length > 200) requests.value.pop();
        }
    });

    listen("dev://clear-logs", () => {
        requests.value = [];
    });

    if (isRequestsWindow) {
        emit("dev://sync-request");
        listen<RequestLog[]>("dev://sync-response", (event) => {
            const existingIds = new Set(requests.value.map(r => r.id));
            const newLogs = event.payload.filter(r => !existingIds.has(r.id));
            requests.value = [...requests.value, ...newLogs].sort((a, b) => b.timestamp - a.timestamp);
        });
    } else {
        listen("dev://sync-request", () => {
            emit("dev://sync-response", requests.value);
        });

        // Backend requests from Rust's perform_backend_request come in via a
        // separate channel so they're routed through addRequest() here in the
        // main window, giving them the same reliable frontend re-broadcast that
        // frontend HTTP requests receive (and ensuring the Request Viewer window
        // sees them even if its listener wasn't fully registered yet).
        listen<Omit<RequestLog, "timestamp">>("dev://backend-request", (event) => {
            addRequest(event.payload);
        });
    }

    watch(isDevMode, (value) => {
        localStorage.setItem("devMode", value.toString());
    });

    function addRequest(log: Omit<RequestLog, "timestamp">) {
        const fullLog: RequestLog = {
            ...log,
            timestamp: Date.now(),
        };

        requests.value.unshift(fullLog);
        emit("dev://request", fullLog);

        if (requests.value.length > 200) {
            requests.value.pop();
        }
    }

    function clearRequests() {
        requests.value = [];
        emit("dev://clear-logs");
    }

    return {
        isDevMode,
        requests,
        addRequest,
        clearRequests,
    };
});
