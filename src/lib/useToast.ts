import { reactive } from "vue";

export type ToastVariant = "success" | "error" | "info" | "warning";

export interface Toast {
    id: number;
    message: string;
    variant: ToastVariant;
    duration: number;
}

let _nextId = 0;

const toasts = reactive<Toast[]>([]);

function push(message: string, variant: ToastVariant = "info", duration = 4000) {
    const id = _nextId++;
    toasts.push({ id, message, variant, duration });

    if (duration > 0) {
        setTimeout(() => dismiss(id), duration);
    }
}

function dismiss(id: number) {
    const idx = toasts.findIndex((t) => t.id === id);
    if (idx !== -1) toasts.splice(idx, 1);
}

export function useToast() {
    return {
        toasts,
        push,
        dismiss,
        success: (msg: string, dur?: number) => push(msg, "success", dur),
        error: (msg: string, dur?: number) => push(msg, "error", dur),
        info: (msg: string, dur?: number) => push(msg, "info", dur),
        warning: (msg: string, dur?: number) => push(msg, "warning", dur),
    };
}
