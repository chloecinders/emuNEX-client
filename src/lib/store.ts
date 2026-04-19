import { appDataDir, join } from "@tauri-apps/api/path";
import { mkdir } from "@tauri-apps/plugin-fs";
import { load, type Store } from "@tauri-apps/plugin-store";

let globalStore: Store | null = null;
const domainStores: Record<string, Store> = {};

export async function getGlobalStore(): Promise<Store> {
    if (!globalStore) {
        globalStore = await load("store.json");
    }
    return globalStore;
}

export function normalizeDomain(domain: string): string {
    try {
        const hasProtocol = domain.includes("://");
        let protocol = "https://";

        const plainDomain = hasProtocol ? domain.split("://")[1] : domain;
        if (
            plainDomain.startsWith("127.0.0.1") ||
            plainDomain.startsWith("localhost") ||
            plainDomain
                .split(":")[0]
                .split(".")
                .every((part) => !isNaN(Number(part)))
        ) {
            protocol = "http://";
        }

        const url = new URL(hasProtocol ? domain : `${protocol}${domain}`);
        let origin = url.origin.toLowerCase();

        if (url.hostname === "localhost") {
            origin = origin.replace("localhost", "127.0.0.1");
        }

        return origin;
    } catch (e) {
        return domain.toLowerCase().trim();
    }
}

export async function getDomainFolder(domain: string): Promise<string> {
    const normalized = normalizeDomain(domain);
    const base = await appDataDir();
    const folderName = normalized.replace(/[^a-z0-9]/gi, "_").toLowerCase();
    const domainFolder = await join(base, "domains", folderName);
    return domainFolder;
}

export async function getDomainStore(domain: string): Promise<Store> {
    const normalized = normalizeDomain(domain);
    if (!domainStores[normalized]) {
        const folder = await getDomainFolder(normalized);
        try {
            await mkdir(folder, { recursive: true });
        } catch (e) { }
        domainStores[normalized] = await load(await join(folder, "store.json"));
    }
    return domainStores[normalized];
}

export async function getCurrentDomain(): Promise<string | null> {
    const store = await getGlobalStore();
    const domain = await store.get<string>("domain");
    return domain ?? null;
}

export async function getStore(): Promise<Store> {
    const domain = await getCurrentDomain();
    if (domain) {
        return await getDomainStore(domain);
    }
    return await getGlobalStore();
}

export async function getSavedDomains(): Promise<string[]> {
    const store = await getGlobalStore();
    return (await store.get<string[]>("domains")) || [];
}

export async function addSavedDomain(domain: string) {
    const normalized = normalizeDomain(domain);
    const store = await getGlobalStore();
    const domains = await getSavedDomains();
    if (!domains.includes(normalized)) {
        domains.push(normalized);
        await store.set("domains", domains);
        await store.save();
    }
}
