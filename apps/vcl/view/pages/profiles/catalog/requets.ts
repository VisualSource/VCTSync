import { gzipDecodeSync } from "iced:compression";
import type { CatalogMod } from "./types.js";

const INDEX_URL = "https://thunderstore.io/c/void-crew/api/v1/package-listing-index/";

async function fetchGzipJson<T>(url: string): Promise<T> {
    const response = await fetch(url);
    if (!response.ok) throw new Error(`${url}: ${response.statusText}`);
    return JSON.parse(new TextDecoder().decode(gzipDecodeSync(await response.bytes()))) as T;
}

export async function fetchCatalog(): Promise<CatalogMod[]> {
    const chunks = await fetchGzipJson<string[]>(INDEX_URL);

    const list = await Promise.all(chunks.map(url => fetchGzipJson<CatalogMod[]>(url)));

    return list.flat().filter(mod => !mod.is_deprecated);
}