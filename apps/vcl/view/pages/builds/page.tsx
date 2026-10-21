import { useEffect, useState } from "react";
import { Center } from "../../utils.js";
import { useMutation, useQuery } from "@tanstack/react-query";
import { invoke, listen } from "react-iced-native";
import { nanoid } from "nanoid";

import Network from "../../assets/icons/network.svg" with { type: "svg" };
import FlaskConical from "../../assets/icons/flask-conical.svg" with { type: "svg" };
import Download from "@icon/download.svg" with { type: "svg" };
import Check from "@icon/check.svg" with { type: "svg" };
import Refresh from "@icon/refresh-cw.svg" with { type: "svg" };
import Trash from "@icon/trash.svg" with { type: "svg" };
import WifiOff from "@icon/wifi-off.svg" with { type: "svg" };
import TriangleAlert from "@icon/triangle-alert.svg" with { type: "svg" };
import ArrowLeftRight from "@icon/arrow-left-right.svg" with { type: "svg" };

import type { GithubVersion, Version } from "./types.js";
import { useApp } from "../../state.js";
import { useProfile, useProfileCollection, useProfileOptions } from "../profiles/query.js";

const MUTED = "background.strongest.text";

/** Builds install like a Thunderstore mod with this author/name (BUILD_FULL_NAME on the host). */
const BUILD_AUTHOR = "VisualSource";
const BUILD_NAME = "VoidCrewTerminus";

const rtf = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

const formatDiff = (diff: Temporal.Duration) => {
    let value = -diff.minutes;
    let unit: Intl.RelativeTimeFormatUnit = "minute";

    if (diff.days !== 0) {
        value = -diff.days;
        unit = "day";
    } else if (diff.hours !== 0) {
        value = -diff.hours;
        unit = "hour"
    }

    return rtf.format(value, unit)
}

const mb = (bytes: number) => `${(bytes / 1_048_576).toFixed(1)} MB`;

const remoteQueryFn = async ({ signal }: { signal: AbortSignal }) => {
    let response = await fetch("https://api.github.com/repos/VisualSource/VoidCrewTerminus/releases", {
        headers: {
            "Accept": "application/vnd.github+json"
        },
        signal,
    });

    if (!response.ok) throw new Error(`failed to load remote versions: ${response.statusText}`);

    const request = await response.json() as GithubVersion[];
    const now = Temporal.Now.plainDateTimeISO("UTC");

    // Drafts and releases without a zip asset can't be installed.
    return request.filter(e => !e.draft && e.assets[0]).map(e => ({
        id: e.assets[0]!.node_id,
        version: e.tag_name,
        timestamp: formatDiff(Temporal.PlainDateTime.from(e.published_at.replace("Z", "")).until(now, { largestUnit: "days" })),
        content_type: "remote",
        source_url: e.assets[0]!.browser_download_url,
        digest: e.assets[0]!.digest,
        size: e.assets[0]!.size,
        branch: e.target_commitish,
        prerelease: e.prerelease,
    } satisfies Version));
}

export const localQueryFn = async ({ signal }: { signal: AbortSignal, }) => {
    const app = useApp.getState();

    if (!app.devApiUrl) throw new Error("No api url configured");
    if (!app.devApiToken) throw new Error("No api token configured");

    const response = await fetch(`http://${app.devApiUrl}/versions`, {
        headers: {
            "Authorization": `Bearer ${app.devApiToken}`
        },
        signal,
    });
    if (!response.ok) throw new Error(`failed to load local versions: ${response.statusText}`);

    const request = await response.json() as Version[];
    const now = Temporal.Now.plainDateTimeISO("UTC");

    return request.map(e => ({
        ...e,
        timestamp: formatDiff(Temporal.PlainDateTime.from(e.timestamp.replace("Z", "")).until(now, { largestUnit: "days" })),
    }))
}

type Progress = { id: string, downloaded: number, total: number };

const Chip = ({ label, palette, mono }: { label: string, palette: string, mono?: boolean }) => (
    <view padding={[1, 6]} style={{ background: palette, borderRadius: 9 }}>
        <text size={11} color={`${palette}.text`} font={mono ? "monospace" : "default"}>{label}</text>
    </view>
);

const Badge = ({ dev }: { dev: boolean }) => (
    <view alignX="center" alignY="center" width={34} height={34} style={{ background: dev ? "#7a5bd1" : "#3a7bc8", borderRadius: 4 }}>
        <svg height={15} width={15} color="#ffffff" src={dev ? FlaskConical : Network} />
    </view>
);

const Empty = ({ icon, title, detail, action, onAction }: { icon: SvgHandle, title: string, detail?: string, action?: string, onAction?: () => void }) => (
    <Center spacing={8}>
        <svg height={40} width={40} color={MUTED} src={icon} />
        <text center>{title}</text>
        {detail ? <text size={12} color={MUTED} center>{detail}</text> : null}
        {action && onAction ? (
            <button width="shrink" style="secondary" onPress={onAction}>
                <text size={13}>{action}</text>
            </button>
        ) : null}
    </Center>
);

export const Builds = () => {
    const app = useApp();
    const [source, setSource] = useState<"remote" | "local">("remote");
    const [chosenProfile, setChosenProfile] = useState<string | null>(null);
    const [progress, setProgress] = useState<Progress | null>(null);

    const targetId = chosenProfile ?? app.activeProfile;
    const options = useProfileOptions();
    const { data: profile } = useProfile(targetId);
    const collection = useProfileCollection();

    const agentConfigured = !!app.devApiUrl && !!app.devApiToken;
    const remote = useQuery({ queryKey: ["builds::remote"], queryFn: remoteQueryFn, staleTime: 60_000 });
    const local = useQuery({
        queryKey: ["builds::local", app.devApiUrl, app.devApiToken],
        queryFn: localQueryFn,
        enabled: agentConfigured,
        retry: false,
    });
    const builds = source === "remote" ? remote : local;
    const latestId = remote.data?.find(b => !b.prerelease)?.id;

    const installedMod = profile?.mods.find(m => m.author === BUILD_AUTHOR && m.name === BUILD_NAME);
    const installedSource = installedMod?.source === "dev" ? "local" : "remote";
    const installedBuild = installedMod
        ? (installedSource === "local" ? local.data : remote.data)?.find(b => b.version === installedMod.version)
        : undefined;

    useEffect(() => listen<Progress>("build::progress", setProgress), []);

    const install = useMutation({
        mutationFn: async (build: Version) => {
            if (!profile || !targetId) throw new Error("Pick a profile to install into.");
            setProgress({ id: build.id, downloaded: 0, total: build.size ?? 0 });

            const result = await invoke<{ error?: string, icon?: string | null }>("install::build", {
                id: build.id,
                profile_dir: profile.dir,
                source_url: build.source_url,
                digest: build.digest ?? "",
            });
            if (result.error) throw new Error(result.error);

            const record = {
                name: BUILD_NAME,
                author: BUILD_AUTHOR,
                version: build.version,
                icon: result.icon ?? "",
                description: build.branch,
                dependencies: [],
                source: build.content_type === "local" ? "dev" as const : "release" as const,
            };
            collection.update(targetId, draft => {
                const existing = draft.mods.find(m => m.author === BUILD_AUTHOR && m.name === BUILD_NAME);
                if (existing) Object.assign(existing, record, { enabled: true });
                else draft.mods.push({ ...record, id: nanoid(), enabled: true });
            });
        },
        onSettled: () => setProgress(null),
    });

    const uninstall = useMutation({
        mutationFn: async () => {
            if (!profile || !targetId) return;
            const result = await invoke<{ error?: string }>("mods::uninstall", { profile_dir: profile.dir, full_name: `${BUILD_AUTHOR}-${BUILD_NAME}` });
            if (result.error) throw new Error(result.error);
            collection.update(targetId, draft => {
                draft.mods = draft.mods.filter(m => !(m.author === BUILD_AUTHOR && m.name === BUILD_NAME));
            });
        },
    });

    const installingId = install.isPending ? install.variables?.id : undefined;
    const failedId = install.isError ? install.variables?.id : undefined;
    const checkedMin = Math.round((Date.now() - builds.dataUpdatedAt) / 60_000);

    const progressText = (p: Progress | null) => {
        if (!p || p.downloaded === 0) return "Starting download…";
        if (p.total && p.downloaded >= p.total) return `Verifying and installing into ${profile?.name ?? "profile"}…`;
        return p.total ? `Downloading ${mb(p.downloaded)} of ${mb(p.total)}` : `Downloading ${mb(p.downloaded)}`;
    };

    return (
        <col height="fill">
            <row padding={[10, 16, 8, 12]} alignY="center">
                <text size={15} font={{ weight: "semibold" }} width="fill">Versions</text>
                <button width="shrink" style="subtle" disabled={builds.isFetching} onPress={() => { void remote.refetch(); if (agentConfigured) void local.refetch(); }}>
                    <svg height={15} width={15} color="background.text" src={Refresh} />
                </button>
            </row>

            <col spacing={8} padding={[0, 12, 10, 12]}>
                <row spacing={8} alignY="center">
                    <text size={12} color={MUTED}>Install to</text>
                    {options.length ? (
                        <select width="fill" options={options} placeholder="Select profile" value={targetId ?? undefined} onSelect={(ev) => setChosenProfile(ev.value)} />
                    ) : (
                        <text size={12} color={MUTED}>No profiles yet. Create one on the Profiles page.</text>
                    )}
                </row>

                <view width="fill" style={{ background: "background.weaker", borderRadius: 3 }} padding={10}>
                    {installedMod ? (
                        <col spacing={6}>
                            <row spacing={8} alignY="center">
                                <Badge dev={installedMod.source === "dev"} />
                                <col width="fill" spacing={2}>
                                    <row spacing={6} alignY="center">
                                        <text size={16} font="monospace">{installedMod.version}</text>
                                        {installedMod.description && installedMod.description !== "main" ? <Chip label={installedMod.description} palette="#3c3550" mono /> : null}
                                    </row>
                                    <text size={12} color={MUTED}>{`${installedMod.source === "dev" ? "Dev build" : "Release"}${installedBuild ? ` · published ${installedBuild.timestamp}` : ""}`}</text>
                                </col>
                                <button width="shrink" style="subtle" disabled={uninstall.isPending || install.isPending} onPress={() => uninstall.mutate()}>
                                    <svg height={15} width={15} color="background.text" src={Trash} />
                                </button>
                            </row>
                            {uninstall.error ? <text size={12} color="danger">{uninstall.error.message}</text> : null}
                        </col>
                    ) : (
                        <row spacing={8} alignY="center">
                            <view alignX="center" alignY="center" width={34} height={34} style={{ background: "background.weak", borderRadius: 4 }}>
                                <svg height={15} width={15} color={MUTED} src={Download} />
                            </view>
                            <col spacing={2}>
                                <text>{profile ? "Not installed in this profile" : "No profile selected"}</text>
                                <text size={12} color={MUTED}>Pick a version below to install it.</text>
                            </col>
                        </row>
                    )}
                </view>
            </col>

            <row spacing={2} padding={[0, 12]}>
                {([["remote", "Releases", Network, remote.data?.length], ["local", "Dev builds", FlaskConical, local.data?.length]] as const).map(([key, label, icon, count]) => (
                    <button key={key} width="fill" style={source === key ? "primary" : "secondary"} onPress={() => setSource(key)}>
                        <view width="fill" alignX="center">
                            <row spacing={6} alignY="center">
                                <svg height={15} width={15} color={source === key ? "primary.text" : "secondary.text"} src={icon} />
                                <text size={13}>{count !== undefined ? `${label} ${count}` : label}</text>
                            </row>
                        </view>
                    </button>
                ))}
            </row>

            {source === "local" && !agentConfigured ? (
                <Empty icon={FlaskConical} title="Dev builds come from the dev agent." detail="Set its address and token in Settings." action="Open Settings" onAction={() => app.setPage("settings")} />
            ) : builds.isLoading ? (
                <Center><text color={MUTED}>Checking for versions…</text></Center>
            ) : builds.isError ? (
                <Empty icon={WifiOff} title={source === "local" ? `Couldn't reach the dev agent at ${app.devApiUrl}.` : "Couldn't reach GitHub."} detail={builds.error.message} action="Retry" onAction={() => void builds.refetch()} />
            ) : !builds.data?.length ? (
                <Empty icon={Download} title="No versions published yet." />
            ) : (
                <scroll height="fill" spacing={4}>
                    <col spacing={4} padding={[8, 12, 12, 12]}>
                        {builds.data.map(build => {
                            const isInstalled = !!installedMod && installedMod.version === build.version && installedSource === source;
                            const isInstalling = installingId === build.id;
                            return (
                                <view key={build.id} padding={8} style={{
                                    background: "background.weaker",
                                    borderRadius: 3,
                                    borderWidth: isInstalled ? 1 : undefined,
                                    borderColor: isInstalled ? "success" : undefined,
                                }}>
                                    <col spacing={6}>
                                        <row spacing={8} alignY="center">
                                            <col width="fill" spacing={2}>
                                                <row spacing={6} alignY="center">
                                                    <text font="monospace">{build.version}</text>
                                                    {build.prerelease ? <Chip label="Pre-release" palette="warning" /> : null}
                                                    {build.branch && build.branch !== "main" ? <Chip label={build.branch} palette="#3c3550" mono /> : null}
                                                    {build.id === latestId && source === "remote" ? <Chip label="Latest" palette="background.weak" /> : null}
                                                </row>
                                                <text size={12} color={MUTED}>{build.size ? `${build.timestamp} · ${mb(build.size)}` : build.timestamp}</text>
                                            </col>

                                            {isInstalling ? (
                                                <button width="shrink" disabled><text size={13}>Installing…</text></button>
                                            ) : isInstalled ? (
                                                <row spacing={4} alignY="center">
                                                    <Chip label="Installed" palette="success" />
                                                    <button width="shrink" style="text" disabled={install.isPending} onPress={() => install.mutate(build)}>
                                                        <text size={13}>Reinstall</text>
                                                    </button>
                                                </row>
                                            ) : (
                                                <button width="shrink" style={installedMod ? "secondary" : "primary"} disabled={install.isPending || !profile} onPress={() => install.mutate(build)}>
                                                    <row spacing={4} alignY="center">
                                                        <svg height={12} width={12} color={installedMod ? "secondary.text" : "primary.text"} src={installedMod ? ArrowLeftRight : Download} />
                                                        <text size={13}>{installedMod ? "Switch" : "Install"}</text>
                                                    </row>
                                                </button>
                                            )}
                                        </row>

                                        {isInstalling ? (
                                            <col spacing={3}>
                                                <text size={12} color={MUTED}>{progressText(progress?.id === build.id ? progress : null)}</text>
                                                <progress
                                                    value={progress?.id === build.id && progress.total ? progress.downloaded : 0}
                                                    max={progress?.id === build.id && progress.total ? progress.total : 1}
                                                    height={6}
                                                />
                                            </col>
                                        ) : failedId === build.id && install.error ? (
                                            <row spacing={6} alignY="center">
                                                <svg height={15} width={15} color="danger" src={TriangleAlert} />
                                                <text size={12} color="danger" width="fill">{install.error.message}</text>
                                                <button width="shrink" style="text" onPress={() => install.mutate(build)}>
                                                    <text size={13}>Retry</text>
                                                </button>
                                            </row>
                                        ) : null}
                                    </col>
                                </view>
                            );
                        })}
                    </col>
                </scroll>
            )}

            <hr />
            <row padding={[6, 16, 6, 12]}>
                <text size={12} color={MUTED}>{
                    source === "remote" ? `github.com/VisualSource/VoidCrewTerminus${remote.data ? ` · checked ${checkedMin < 1 ? "just now" : `${checkedMin} min ago`}` : ""}`
                        : agentConfigured ? `Dev agent ${app.devApiUrl}${local.data ? ` · checked ${checkedMin < 1 ? "just now" : `${checkedMin} min ago`}` : ""}`
                            : "Dev agent not configured"
                }</text>
            </row>
        </col>
    );
}
