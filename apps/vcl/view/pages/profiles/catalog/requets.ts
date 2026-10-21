import { gzipDecodeSync } from "iced:compression";
import { invoke } from "react-iced-native";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { nanoid } from "nanoid";
import { useProfile, useProfileCollection } from "../query.js";
import { useProfileState } from "../state.js";
import type { CatalogMod, ZipManifest } from "./types.js";
import type { Mod } from "../types.js";

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

export const useCatalog = () => useQuery({
    queryKey: ["voidcrew-mod-list"],
    queryFn: fetchCatalog,
    staleTime: 5 * 60_000,
    select: mods => ({ mods, byName: new Map(mods.map(mod => [mod.full_name, mod])) }),
});

/** "Owner-Name-1.2.3" -> "Owner-Name" */
export const depKey = (dep: string) => dep.split("-").slice(0, 2).join("-");
export const depVersion = (dep: string) => dep.split("-")[2] ?? "";
export const pretty = (name: string) => name.replace(/_/g, " ");
export const totalDownloads = (mod: CatalogMod) => mod.versions.reduce((sum, v) => sum + v.downloads, 0);

export const formatCount = (n: number) => n >= 1e6 ? `${(n / 1e6).toFixed(1)}M` : n >= 1e3 ? `${Math.round(n / 1e3)}k` : String(n);

const cmpVersion = (a: string, b: string) => {
    const x = a.split(".").map(Number), y = b.split(".").map(Number);
    for (let i = 0; i < 3; i++) if ((x[i] || 0) !== (y[i] || 0)) return (x[i] || 0) - (y[i] || 0);
    return 0;
}

export type InstallStatus = "none" | "installed" | "outdated";

/** Installed versions of the open profile's mods, keyed "Author-Name". */
export const useInstalled = () => {
    const profileId = useProfileState(state => state.detailId);
    const { data } = useProfile(profileId);
    return new Map(data?.mods.map(mod => [`${mod.author}-${mod.name}`, mod.version]) ?? []);
}

export const statusOf = (installed: Map<string, string>, key: string, latest: string): InstallStatus => {
    const version = installed.get(key);
    if (!version) return "none";
    return cmpVersion(version, latest) < 0 ? "outdated" : "installed";
}

/** Catalog packages needed by `deps` (transitively) that the profile doesn't have. */
export const missingDeps = (deps: string[], installed: Map<string, string>, byName: Map<string, CatalogMod>, seen = new Map<string, CatalogMod>()) => {
    for (const dep of deps) {
        const key = depKey(dep);
        const mod = byName.get(key);
        if (installed.has(key) || seen.has(key) || !mod) continue;
        seen.set(key, mod);
        missingDeps(mod.versions[0]?.dependencies ?? [], installed, byName, seen);
    }
    return [...seen.values()];
}

/** A mod to install: its profile record plus where the host gets the files. */
export type ModInstall = Omit<Mod, "id" | "enabled"> & { path?: string; url?: string };

export const fromCatalog = (mod: CatalogMod): ModInstall => ({
    name: mod.name,
    author: mod.owner,
    version: mod.versions[0]?.version_number ?? "",
    icon: "",
    description: mod.versions[0]?.description ?? "",
    dependencies: mod.versions[0]?.dependencies ?? [],
    source: "thunderstore",
    url: mod.versions[0]?.download_url ?? "",
});

/** Installs mods one at a time (pass dependencies first) and records each on the open profile as it lands. */
export const useInstallMods = () => {
    const profileId = useProfileState(state => state.detailId);
    const { data: profile } = useProfile(profileId);
    const collection = useProfileCollection();
    const [step, setStep] = useState("");

    const mutation = useMutation({
        mutationFn: async (mods: ModInstall[]) => {
            if (!profileId || !profile) throw new Error("No profile open.");

            for (const [i, { path, url, ...mod }] of mods.entries()) {
                setStep(`Installing ${pretty(mod.name)} (${i + 1}/${mods.length})`);
                const result = await invoke<{ error?: string, icon?: string | null }>("mods::install", {
                    profile_dir: profile.dir,
                    full_name: `${mod.author}-${mod.name}`,
                    path,
                    url,
                });
                if (result.error) throw new Error(`${pretty(mod.name)}: ${result.error}`);
                mod.icon = result.icon ?? "";

                collection.update(profileId, draft => {
                    const existing = draft.mods.find(m => m.author === mod.author && m.name === mod.name);
                    // A reinstall writes fresh (enabled) files, so the record must say enabled too.
                    if (existing) Object.assign(existing, mod, { enabled: true });
                    else draft.mods.push({ ...mod, id: nanoid(), enabled: true });
                });
            }
        },
    });

    return { ...mutation, step };
}

export const pickZip = () => invoke<{ path: string | null }>("open-file", { filters: ["zip"] });

export const readZipManifest = async (path: string) => {
    const result = await invoke<ZipManifest | { error: string }>("zip::read-manifest", { path });
    if ("error" in result) throw new Error(result.error);
    return result;
}
