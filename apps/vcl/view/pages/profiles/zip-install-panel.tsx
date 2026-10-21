import BackArrow from "@icon/arrow-left.svg";
import PackagePlus from "@icon/package-plus.svg";
import TriangleAlert from "@icon/triangle-alert.svg";
import { useQuery } from "@tanstack/react-query";
import { useEffect } from "react";
import { useProfileState } from "./state.js";
import { ProfileIcon } from "./profile-icon.js";
import { DependencyList } from "./catalog/mod-details.js";
import { fromCatalog, missingDeps, pickZip, readZipManifest, useCatalog, useInstallMods, useInstalled } from "./catalog/requets.js";

const MUTED = "background.strongest.text";

const Note = ({ children, color = "warning" }: { children: string, color?: string }) => (
    <row spacing={6}>
        <svg height={15} width={15} color={color} src={TriangleAlert} />
        <text size={12} color={color} width="fill">{children}</text>
    </row>
);

export const ZipInstallPanel = () => {
    const path = useProfileState(state => state.zipPath);
    const close = useProfileState(state => state.closeCatalogSubmenu);
    const openZipInstall = useProfileState(state => state.openZipInstall);
    const toggleCatalog = useProfileState(state => state.toggleCatlog);
    const { data: catalog } = useCatalog();
    const installed = useInstalled();
    const { mutate, isPending, error: installError, step, reset } = useInstallMods();
    useEffect(() => reset(), [path]);

    const { data: manifest, isLoading, isError, error } = useQuery({
        queryKey: ["zip-manifest", path],
        enabled: !!path,
        queryFn: () => readZipManifest(path!),
        retry: false,
    });

    if (!path) return null;

    const file = path.split(/[\\/]/).at(-1) ?? path;
    // r2modman convention: "Author-Name-Version.zip"
    const parts = file.replace(/\.zip$/i, "").split("-");
    const author = manifest?.author ?? (parts.length >= 3 ? parts[0] : undefined);
    const missing = manifest && catalog ? missingDeps(manifest.dependencies, installed, catalog.byName) : [];

    const chooseAnother = () => pickZip().then(result => {
        if (result.path) openZipInstall(result.path);
    }).catch(err => console.error(err));

    const add = () => {
        if (!manifest) return;
        mutate([...missing.map(fromCatalog), {
            name: manifest.name,
            author: author ?? "local",
            version: manifest.version_number,
            icon: "",
            description: manifest.description,
            dependencies: manifest.dependencies,
            source: "local",
            path,
        }], { onSuccess: () => toggleCatalog(false) });
    };

    return (
        <opaque>
            <view style={{ background: "background" }} height="fill">
                <col spacing={8} padding={6}>
                    <row spacing={6} alignY="center">
                        <button width="shrink" style="subtle" onPress={close}>
                            <svg height={15} width={15} color="background.text" src={BackArrow} />
                        </button>
                        <col>
                            <text size={14} font={{ weight: "semibold" }}>Add from file</text>
                            <text size={12} color={MUTED}>{file}</text>
                        </col>
                    </row>

                    <scroll height="fill" spacing={4}>
                        {isLoading ? (
                            <text color={MUTED}>Reading manifest…</text>
                        ) : isError || !manifest ? (
                            <Note color="danger">{`${file} isn't a valid mod package: ${error?.message ?? "manifest.json was not found at the root of the zip."}`}</Note>
                        ) : (
                            <col spacing={8}>
                                <row spacing={10} alignY="center">
                                    <ProfileIcon type="sword" />
                                    <col spacing={2}>
                                        <text size={16} font={{ weight: "semibold" }}>{manifest.name}</text>
                                        <text size={12} color={MUTED}>{`v${manifest.version_number}`}</text>
                                    </col>
                                </row>

                                <row spacing={12}>
                                    <col spacing={4}>
                                        <text size={13} color={MUTED}>Author</text>
                                        <text size={13} color={MUTED}>Website</text>
                                    </col>
                                    <col spacing={4} width="fill">
                                        <text size={13}>{author ? `${author} (from ${manifest.author ? "manifest" : "file name"})` : "Unknown"}</text>
                                        <text size={13}>{manifest.website_url || "—"}</text>
                                    </col>
                                </row>

                                <text>{manifest.description}</text>

                                {!author ? <Note>{`No author found, so update checks won't work. Rename to Author-${manifest.name}-${manifest.version_number}.zip to fix.`}</Note> : null}
                                {author && installed.has(`${author}-${manifest.name}`) ? <Note>Already in this profile — it will be replaced.</Note> : null}

                                <DependencyList deps={manifest.dependencies} />
                            </col>
                        )}
                    </scroll>

                    <hr />
                    {manifest ? (
                        <text size={12} color={MUTED}>{isPending ? step : missing.length ? "Missing dependencies will be installed from Thunderstore." : "All dependencies present."}</text>
                    ) : null}
                    {installError ? <text size={12} color="danger">{installError.message}</text> : null}
                    <row spacing={4}>
                        <button style="secondary" width="shrink" disabled={isPending} onPress={chooseAnother}>
                            <text size={14}>Choose another…</text>
                        </button>
                        <button disabled={!manifest || isPending} onPress={add}>
                            <view width="fill" alignX="center">
                                <row spacing={6} alignY="center">
                                    <svg height={15} width={15} color="primary.text" src={PackagePlus} />
                                    <text size={14}>{isPending ? "Installing…" : `Add to profile${missing.length ? ` (+${missing.length})` : ""}`}</text>
                                </row>
                            </view>
                        </button>
                    </row>
                </col>
            </view>
        </opaque>
    );
}
