import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { useToast } from "../lib/useToast";

export type DownloadStatus = "queued" | "downloading" | "done" | "error" | "cancelled";

export type DownloadKind =
    | { kind: "rom"; game_id: string; console: string }
    | { kind: "emulator"; console: string };

export interface DownloadItem {
    id: string;
    label: string;
    kind: DownloadKind;
    status: DownloadStatus;
    total_bytes: number;
    downloaded_bytes: number;
    speed_bps: number;
    queued_at: number;
    started_at?: number;
    completed_at?: number;
    error?: string;
    extraction_progress?: number;
}

export const useDownloadStore = defineStore("downloadStore", () => {
    const items = ref<DownloadItem[]>([]);
    const toast = useToast();

    const activeCount = computed(
        () => items.value.filter((i) => i.status === "queued" || i.status === "downloading").length,
    );

    const hasActive = computed(() => activeCount.value > 0);

    async function init() {
        const queue = await invoke<DownloadItem[]>("get_download_queue");
        items.value = queue;

        await listen<DownloadItem>("download-queued", (event) => {
            const existing = items.value.findIndex((i) => i.id === event.payload.id);
            if (existing === -1) {
                items.value.push(event.payload);
            }
        });

        await listen<{ id: string }>("download-started", (event) => {
            const item = items.value.find((i) => i.id === event.payload.id);
            if (item) {
                item.status = "downloading";
                item.started_at = Date.now();
            }
        });

        await listen<{ id: string; downloaded: number; total: number; speed_bps: number }>(
            "download-progress",
            (event) => {
                const item = items.value.find((i) => i.id === event.payload.id);
                if (item) {
                    item.downloaded_bytes = event.payload.downloaded;
                    item.total_bytes = event.payload.total || item.total_bytes;
                    item.speed_bps = event.payload.speed_bps;
                }
            },
        );

        await listen<{ id: string; kind: { type: string; game_id?: string } }>(
            "download-complete",
            (event) => {
                const item = items.value.find((i) => i.id === event.payload.id);
                if (item) {
                    item.status = "done";
                    item.downloaded_bytes = item.total_bytes;
                    item.extraction_progress = 100;
                    item.completed_at = Date.now();

                    if (event.payload.kind?.type === "rom") {
                        import("./GameStore").then(({ useGameStore }) => {
                            useGameStore().fetchInstalledGames();
                        });
                    } else if (event.payload.kind?.type === "emulator") {
                        import("./EmulatorStore").then(({ useEmulatorStore }) => {
                            useEmulatorStore().fetchEmulators();
                        });
                    }
                }
            },
        );

        await listen<{ id: string; progress: number }>("extraction-progress", (event) => {
            const item = items.value.find((i) => i.id === event.payload.id);
            if (item) {
                item.extraction_progress = event.payload.progress;
            }
        });

        await listen<{ id: string; error: string }>("download-error", (event) => {
            const item = items.value.find((i) => i.id === event.payload.id);
            if (item) {
                if (event.payload.error === "Cancelled") {
                    item.status = "cancelled";
                } else {
                    item.status = "error";
                    item.error = event.payload.error;
                    toast.error(`${item.label}: ${event.payload.error}`);
                }
                item.completed_at = Date.now();
            }
        });
    }

    async function enqueueRom(opts: {
        label: string;
        game_id: string;
        console: string;
        rom_path: string;
        extension: string;
        name?: string;
        zipped?: boolean;
        zipped_entry?: string;
        total_bytes: number;
        md5?: string;
    }): Promise<string> {
        return invoke<string>("queue_download_rom", {
            label: opts.label,
            gameId: opts.game_id,
            console: opts.console,
            romPath: opts.rom_path,
            extension: opts.extension,
            name: opts.name ?? null,
            zipped: opts.zipped ?? null,
            zippedEntry: opts.zipped_entry ?? null,
            totalBytes: opts.total_bytes,
            md5: opts.md5 ?? null,
        });
    }

    async function enqueueEmulator(opts: {
        label: string;
        console: string;
        emulator_id?: string;
        keep_config?: boolean;
        source_server?: string;
        total_bytes: number;
    }): Promise<string> {
        return invoke<string>("queue_download_emulator", {
            label: opts.label,
            console: opts.console,
            emulatorId: opts.emulator_id ?? null,
            keepConfig: opts.keep_config ?? null,
            sourceServer: opts.source_server ?? null,
            totalBytes: opts.total_bytes,
        });
    }

    async function cancel(id: string) {
        const item = items.value.find((i) => i.id === id);
        if (item) {
            item.status = "cancelled";
            item.completed_at = Date.now();
        }
        await invoke("cancel_download", { id });
    }

    async function remove(id: string) {
        await invoke("remove_download", { id });
        items.value = items.value.filter((i) => i.id !== id);
    }

    function isGameQueued(gameId: string): boolean {
        return items.value.some(
            (i) =>
                (i.status === "queued" || i.status === "downloading") &&
                i.kind.kind === "rom" &&
                (i.kind as any).game_id === gameId,
        );
    }

    function isEmulatorQueued(consoleName: string): boolean {
        return items.value.some(
            (i) =>
                (i.status === "queued" || i.status === "downloading") &&
                i.kind.kind === "emulator" &&
                (i.kind as any).console.toLowerCase() === consoleName.toLowerCase()
        );
    }

    function getItemForGame(gameId: string): DownloadItem | undefined {
        const matching = items.value.filter(
            (i) => i.kind.kind === "rom" && (i.kind as any).game_id === gameId,
        );
        return matching.find(i => i.status === "queued" || i.status === "downloading") || matching[0];
    }

    function getEmulatorItem(consoleName: string): DownloadItem | undefined {
        const matching = items.value.filter(
            (i) => i.kind.kind === "emulator" && (i.kind as any).console.toLowerCase() === consoleName.toLowerCase()
        );
        return matching.find(i => i.status === "queued" || i.status === "downloading") || matching[0];
    }

    return {
        items,
        activeCount,
        hasActive,
        init,
        enqueueRom,
        enqueueEmulator,
        cancel,
        remove,
        isGameQueued,
        isEmulatorQueued,
        getItemForGame,
        getEmulatorItem,
    };
});
