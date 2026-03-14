import { invoke } from "@tauri-apps/api/core";
import { useAuthStore } from "../stores/AuthStore";

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

class ApiClient {
    private async request<T>(endpoint: string, method: string, body?: any): V1ApiResponseType<T> {
        const store = useAuthStore();
        const baseUrl = `${store.domain}/api/v1`;
        const url = `${baseUrl}${endpoint}`;

        try {
            const response = await invoke<T>("http", {
                req: {
                    url,
                    method,
                    body,
                    token: store.token || null,
                },
            });

            return response as any;
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

            throw parsedError;
        }
    }

    public get<T>(path: string): V1ApiResponseType<T> {
        return this.request<T>(path, "GET");
    }

    public post<T>(path: string, body: any): V1ApiResponseType<T> {
        return this.request<T>(path, "POST", body);
    }
}

export const http = new ApiClient();

export function useStoragePath(route: string): string {
    const authStore = useAuthStore();
    return `${authStore.domain}${authStore.storagePath}/${route}`;
}
