import { invoke } from "@tauri-apps/api/core";
import { getActivePinia } from "pinia";
import { useAuthStore } from "../stores/AuthStore";
import { useDevStore } from "../stores/DevStore";

export interface V1ApiResponse<T> {
    data: T;
    success: true;
}

export interface V1ApiError {
    error: string;
    code: string;
    success: false;
}

export type V1ApiResponseType<T> = Promise<V1ApiResponse<T>>;

function tryLogRequest(entry: {
    method: string;
    url: string;
    body: any;
    response: any;
    success: boolean;
    duration: number;
}) {
    try {
        const pinia = getActivePinia();
        if (!pinia) return;
        const devStore = useDevStore();
        if (devStore.isDevMode) {
            devStore.addRequest(entry);
        }
    } catch { }
}

class ApiClient {
    private async request<T>(endpoint: string, method: string, body?: any): V1ApiResponseType<T> {
        const store = useAuthStore();
        const baseUrl = `${store.domain}/api/v1`;
        const url = `${baseUrl}${endpoint}`;
        const startTime = Date.now();

        try {
            const response = await invoke<any>("http", {
                req: {
                    url,
                    method,
                    body,
                    token: store.token || null,
                },
            });

            tryLogRequest({ method, url, body, response, success: true, duration: Date.now() - startTime });

            return response;
        } catch (error: any) {
            let parsedError: V1ApiError;

            try {
                const data = JSON.parse(error);
                parsedError = {
                    error: data.error || "API Error",
                    code: data.code || "ApiError",
                    success: false,
                };
            } catch {
                parsedError = {
                    error: error.toString(),
                    code: "NetworkError",
                    success: false,
                };
            }

            tryLogRequest({ method, url, body, response: parsedError, success: false, duration: Date.now() - startTime });

            throw parsedError;
        }
    }

    public get<T>(path: string): V1ApiResponseType<T> {
        return this.request<T>(path, "GET");
    }

    public post<T>(path: string, body: any): V1ApiResponseType<T> {
        return this.request<T>(path, "POST", body);
    }

    public put<T>(path: string, body: any): V1ApiResponseType<T> {
        return this.request<T>(path, "PUT", body);
    }

    public delete<T>(path: string): V1ApiResponseType<T> {
        return this.request<T>(path, "DELETE");
    }
}

export const http = new ApiClient();

export function useStoragePath(route: string): string {
    const authStore = useAuthStore();
    return `${authStore.domain}${authStore.storagePath}${route}`;
}
