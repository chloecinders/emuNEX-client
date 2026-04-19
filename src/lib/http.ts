import { invoke } from "@tauri-apps/api/core";
import { getActivePinia } from "pinia";
import { useAuthStore } from "../stores/AuthStore";
import { useDevStore } from "../stores/DevStore";

export interface V1ApiResponse<T> {
    data: T;
    success: true;
}

export interface V1ApiError {
    message: string;
    code: string;
    success: false;
}

export type V1ApiResponseType<T> = Promise<V1ApiResponse<T> | V1ApiError>;

function tryLogRequest(entry: {
    id: string;
    method: string;
    url: string;
    status: number;
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

            tryLogRequest({
                id: String(response.id),
                status: response.status || 200,
                method,
                url,
                body,
                response: response.data,
                success: true,
                duration: Date.now() - startTime
            });

            return response.data;
        } catch (error: any) {
            let parsedError: V1ApiError;
            let statusCode = 0;
            let rawResponse: any;
            let proxyErr;

            try {
                proxyErr = JSON.parse(error);

                if (proxyErr.status !== undefined && proxyErr.data !== undefined) {
                    statusCode = proxyErr.status;
                    rawResponse = proxyErr.data;
                    const data = proxyErr.data;
                    parsedError = {
                        message: data.message || data.error || (typeof data === 'string' ? data : "API Error"),
                        code: data.code || `HTTP_${statusCode}`,
                        success: false,
                    };
                } else {
                    rawResponse = proxyErr;
                    parsedError = {
                        message: proxyErr.message || proxyErr.error || "API Error",
                        code: proxyErr.code || "ApiError",
                        success: false,
                    };
                }
            } catch {
                rawResponse = error.toString();
                parsedError = {
                    message: error.toString(),
                    code: "NetworkError",
                    success: false,
                };
            }

            tryLogRequest({
                id: String(proxyErr?.id || 0),
                status: statusCode,
                method,
                url,
                body,
                response: rawResponse,
                success: false,
                duration: Date.now() - startTime
            });

            if (parsedError.code === "InvalidToken") {
                const store = useAuthStore();
                store.invalidateSession();
            }

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
