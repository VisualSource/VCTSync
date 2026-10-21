import { useMutation, useQuery } from "@tanstack/react-query";
import { Center } from "../utils.js";
import { invoke } from "react-iced-native";
import type { ReactNode } from "react";
import { useApp } from "../state.js";
import { open } from "../platform/dialog.js";
import { useCatalog } from "./profiles/catalog/requets.js";
import { localQueryFn } from "./builds/page.js";

import FolderInput from "@icon/folder-input.svg" with { type: "svg" };
import ExternalLink from "@icon/external-link.svg" with { type: "svg" };
import Refresh from "@icon/refresh-cw.svg" with { type: "svg" };
import Check from "@icon/check.svg" with { type: "svg" };
import TriangleAlert from "@icon/triangle-alert.svg" with { type: "svg" };

const MUTED = "background.strongest.text";

type AppInfo = { appDir: string, version: string, themes: string[] };
type GameStatus = { steamDir: string | null, gameDir: string | null, modded: boolean, error: string | null };

const openPath = (path: string) => void invoke("open-path", { path }).catch(console.error);

const Section = ({ title, children }: { title: string, children: ReactNode }) => (
    <col spacing={8}>
        <row spacing={6} alignY="center">
            <text size={12} color={MUTED}>{title.toUpperCase()}</text>
            <col width="fill"><hr /></col>
        </row>
        {children}
    </col>
);

const Field = ({ label, hint, children }: { label: string, hint?: string, children: ReactNode }) => (
    <col spacing={4}>
        <text size={13}>{label}</text>
        {children}
        {hint ? <text size={12} color={MUTED}>{hint}</text> : null}
    </col>
);

/** Read-only path box with an "open in Explorer" button. */
const PathRow = ({ path, placeholder }: { path: string | null, placeholder: string }) => (
    <row spacing={4} alignY="center">
        <view width="fill" padding={[6, 8]} style={{ background: "background.weaker", borderRadius: 2 }}>
            <text size={12} font="monospace" wrapping="glyph" color={path ? "background.text" : MUTED}>{path || placeholder}</text>
        </view>
        <button width="shrink" style="subtle" disabled={!path} onPress={() => path && openPath(path)}>
            <svg height={15} width={15} color="background.text" src={ExternalLink} />
        </button>
    </row>
);

const Note = ({ children, color }: { children: string, color: string }) => (
    <row spacing={6}>
        <svg height={15} width={15} color={color} src={TriangleAlert} />
        <text size={12} color={color} width="fill">{children}</text>
    </row>
);

const Chip = ({ label, palette }: { label: string, palette: "success" | "warning" | "background.weak" }) => (
    <view padding={[1, 6]} style={{ background: palette, borderRadius: 9 }}>
        <text size={11} color={`${palette}.text`}>{label}</text>
    </view>
);

const METHOD_HINT = {
    steam: "Starts through Steam with the overlay and playtime tracking.",
    direct: "Runs Void Crew.exe directly. Steam must already be running.",
};

export const Settings = () => {
    const settings = useApp();
    const update = settings.updateSettings;
    const method = settings.launchMethod || "steam";

    const { data: info } = useQuery({
        queryKey: ["app-info"],
        queryFn: () => invoke<AppInfo>("app::info", {}),
        staleTime: Infinity,
    });

    // Waits for the debounced save so the host locates Steam with the folder just typed.
    const status = useQuery({
        queryKey: ["game-status", settings.steamDir],
        queryFn: () => invoke<GameStatus>("game::status", {}),
        enabled: settings.loaded && settings.saveState === "saved",
    });

    const unlink = useMutation({
        mutationFn: async () => {
            const result = await invoke<{ error?: string }>("game::unlink", {});
            if (result.error) throw new Error(result.error);
        },
        onSettled: () => status.refetch(),
    });

    const agent = useMutation({
        mutationFn: () => localQueryFn({ signal: new AbortController().signal }),
    });

    const catalog = useCatalog();
    const syncedMin = Math.round((Date.now() - catalog.dataUpdatedAt) / 60_000);

    if (!settings.loaded) return (
        <Center><text color={MUTED}>Loading settings…</text></Center>
    );

    const game = status.data;

    return (
        <col height="fill">
            <row padding={[10, 16, 8, 12]} alignY="center">
                <text size={15} font={{ weight: "semibold" }}>Settings</text>
                <space width="fill" />
                {settings.saveState === "error" ? (
                    <text size={12} color="danger">{`Not saved: ${settings.saveError}`}</text>
                ) : (
                    <row spacing={4} alignY="center">
                        {settings.saveState === "saved" ? <svg height={12} width={12} color={MUTED} src={Check} /> : null}
                        <text size={12} color={MUTED}>{settings.saveState === "saving" ? "Saving…" : "Saved"}</text>
                    </row>
                )}
            </row>
            <hr />

            <scroll height="fill" spacing={4}>
                <col spacing={16} padding={[10, 12, 16, 12]}>
                    <Section title="Appearance">
                        <Field label="Theme">
                            <select
                                width="fill"
                                options={info?.themes ?? []}
                                value={settings.theme || "Dark"}
                                onSelect={(ev) => update({ theme: ev.value })}
                            />
                        </Field>
                    </Section>

                    <Section title="Game">
                        <Field label="Steam folder">
                            <row spacing={4}>
                                <input type="text" size={13} padding={[6, 8]}
                                    placeholder={settings.steamDir ? "" : game?.steamDir ?? "Auto-detect"}
                                    value={settings.steamDir}
                                    onChange={(ev) => update({ steamDir: ev.value })} />
                                <button width="shrink" style="secondary" onPress={() => {
                                    open({ directory: true }).then(result => {
                                        if (result.path) update({ steamDir: result.path });
                                    }).catch(console.error);
                                }}>
                                    <svg height={15} width={15} color="secondary.text" src={FolderInput} />
                                </button>
                            </row>
                            {settings.steamDir ? (
                                <row spacing={6} alignY="center">
                                    <Chip label="Custom" palette="background.weak" />
                                    <button width="shrink" style="text" onPress={() => update({ steamDir: "" })}>
                                        <text size={12}>Reset to detected</text>
                                    </button>
                                </row>
                            ) : game?.steamDir ? (
                                <row><Chip label="Auto-detected" palette="success" /></row>
                            ) : null}
                            {game?.error ? <Note color="danger">{`${game.error}. Pick the folder that contains Steam.exe.`}</Note> : null}
                        </Field>

                        <Field label="Void Crew folder" hint="Found from Steam's library folders. Change the Steam folder if it's wrong.">
                            <PathRow path={game?.gameDir ?? null} placeholder={status.isFetching ? "Looking…" : "Not found"} />
                        </Field>

                        <Field label="Launch method" hint={METHOD_HINT[method]}>
                            <row spacing={2}>
                                <button style={method === "steam" ? "primary" : "secondary"} onPress={() => update({ launchMethod: "steam" })}>
                                    <text size={13} center>Through Steam</text>
                                </button>
                                <button style={method === "direct" ? "primary" : "secondary"} onPress={() => update({ launchMethod: "direct" })}>
                                    <text size={13} center>Run exe directly</text>
                                </button>
                            </row>
                        </Field>

                        <view style={{ background: "background.weaker", borderRadius: 3 }} padding={8}>
                            {!game?.gameDir ? (
                                <text size={12} color={MUTED}>Game folder status is unavailable until the game is found.</text>
                            ) : game.modded ? (
                                <col spacing={6}>
                                    <row spacing={6} alignY="center">
                                        <text width="fill">Game folder has the BepInEx loader</text>
                                        <Chip label="Modded" palette="success" />
                                    </row>
                                    <text size={12} color={MUTED}>winhttp.dll and doorstop_config.ini are in the game folder. Launching a profile updates them.</text>
                                    {unlink.error ? <Note color="danger">{unlink.error.message}</Note> : null}
                                    <row>
                                        <space width="fill" />
                                        <button width="shrink" style="danger" disabled={unlink.isPending} onPress={() => unlink.mutate()}>
                                            <text size={13}>{unlink.isPending ? "Restoring…" : "Restore vanilla game"}</text>
                                        </button>
                                    </row>
                                </col>
                            ) : (
                                <col spacing={6}>
                                    <row spacing={6} alignY="center">
                                        <text width="fill">Game folder is vanilla</text>
                                        <Chip label="Vanilla" palette="background.weak" />
                                    </row>
                                    <text size={12} color={MUTED}>No loader files in the game folder. Launching a profile adds them.</text>
                                </col>
                            )}
                        </view>
                    </Section>

                    <Section title="Storage">
                        <Field label="App folder" hint="Holds settings.ron, profiles.ron and new profiles under profiles\.">
                            <PathRow path={info?.appDir ?? null} placeholder="…" />
                        </Field>

                        <row spacing={10} alignY="center">
                            <col width="fill" spacing={2}>
                                <text>Thunderstore catalog</text>
                                <text size={12} color={MUTED}>{
                                    catalog.isFetching ? "Fetching package list from thunderstore.io…"
                                        : catalog.data ? `${catalog.data.mods.length} packages · synced ${syncedMin < 1 ? "just now" : `${syncedMin} min ago`}`
                                            : "Not synced"
                                }</text>
                            </col>
                            <button width="shrink" style="secondary" disabled={catalog.isFetching} onPress={() => catalog.refetch()}>
                                <row spacing={6} alignY="center">
                                    <svg height={15} width={15} color="secondary.text" src={Refresh} />
                                    <text size={13}>Refresh</text>
                                </row>
                            </button>
                        </row>
                    </Section>

                    <Section title="Logs">
                        <row spacing={10} alignY="center">
                            <col width="fill" spacing={2}>
                                <text>Only show mod log lines</text>
                                <text size={12} color={MUTED}>Hide Unity and game output in the Logs page.</text>
                            </col>
                            <input type="switch" checked={settings.modOnlyLogLines} onChange={(ev) => update({ modOnlyLogLines: ev.value })} />
                        </row>
                    </Section>

                    <Section title="Dev agent">
                        <text size={12} color={MUTED}>Pulls dev builds from a build machine on your network (Builds page).</text>

                        <Field label="Agent address">
                            <row spacing={4}>
                                <input type="text" size={13} padding={[6, 8]} placeholder="192.168.1.94:9787" value={settings.devApiUrl} onChange={(ev) => update({ devApiUrl: ev.value })} />
                                <button width="shrink" style="secondary" disabled={agent.isPending || settings.saveState === "saving"} onPress={() => agent.mutate()}>
                                    <text size={13}>Test</text>
                                </button>
                            </row>
                            {agent.isPending ? <text size={12} color={MUTED}>Connecting…</text>
                                : agent.error ? <Note color="danger">{agent.error.message}</Note>
                                    : agent.data ? (
                                        <row spacing={6} alignY="center">
                                            <Chip label="Connected" palette="success" />
                                            <text size={12} color={MUTED}>{`${agent.data.length} builds available`}</text>
                                        </row>
                                    ) : null}
                        </Field>

                        <Field label="Token">
                            <input type="text" size={13} padding={[6, 8]} placeholder="shared bearer token" value={settings.devApiToken} onChange={(ev) => update({ devApiToken: ev.value })} />
                        </Field>

                        <Field label="Download folder" hint="Empty uses your Downloads folder.">
                            <row spacing={4}>
                                <input type="text" size={13} padding={[6, 8]} placeholder="C:\Users\you\Downloads" value={settings.saveDir} onChange={(ev) => update({ saveDir: ev.value })} />
                                <button width="shrink" style="secondary" onPress={() => {
                                    open({ directory: true }).then(result => {
                                        if (result.path) update({ saveDir: result.path });
                                    }).catch(console.error);
                                }}>
                                    <svg height={15} width={15} color="secondary.text" src={FolderInput} />
                                </button>
                            </row>
                        </Field>
                    </Section>

                    <Section title="About">
                        <row>
                            <text size={12} color={MUTED} width="fill">{`VCL ${info?.version ?? ""} · BepInExPack default 5.4.2305`}</text>
                            <text size={12} color={MUTED}>Reload view: F5</text>
                        </row>
                    </Section>
                </col>
            </scroll>
        </col>
    );
}
