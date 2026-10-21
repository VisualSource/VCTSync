import { useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "react-iced-native";

import FolderIcon from "@icon/folder.svg" with { type: "svg" };
import Download from "@icon/download.svg" with { type: "svg" };
import ScrollText from "@icon/scroll-text.svg" with { type: "svg" };
import TriangleAlert from "@icon/triangle-alert.svg" with { type: "svg" };

import { useApp } from "../../state.js";
import { Center } from "../../utils.js";
import { useProfile, useProfileOptions } from "../profiles/query.js";
import { CORE_SOURCES, parseLog, type Entry, type Level } from "./parse.js";

const MUTED = "background.strongest.text";

/** Each entry is a few iced widgets; past this the newest are kept. */
const MAX_SHOWN = 1000;

type LogFile = { path: string, exists: boolean, size: number, modified: number, content: string | null };
type LogState = Omit<LogFile, "content"> & { entries: Entry[] };

const LEVELS: { level: Level, label: string, palette: string }[] = [
    { level: "error", label: "Errors", palette: "danger" },
    { level: "warning", label: "Warnings", palette: "warning" },
    { level: "info", label: "Info", palette: "background.strong" },
    { level: "debug", label: "Debug", palette: "background.strong" },
];

const LEVEL_TAG: Record<Level, { label: string, color: string }> = {
    error: { label: "ERROR", color: "danger" },
    warning: { label: "WARN", color: "warning" },
    info: { label: "INFO", color: MUTED },
    debug: { label: "DEBUG", color: MUTED },
};

const kb = (bytes: number) => bytes < 1024 * 1024 ? `${(bytes / 1024).toFixed(1)} KB` : `${(bytes / 1_048_576).toFixed(1)} MB`;

const ago = (ms: number) => {
    const s = Math.round((Date.now() - ms) / 1000);
    if (s < 10) return "just now";
    if (s < 60) return `${s}s ago`;
    if (s < 3600) return `${Math.round(s / 60)} min ago`;
    return `${Math.round(s / 3600)} h ago`;
};

const LogEntry = ({ entry, expanded, onToggle }: { entry: Entry, expanded: boolean, onToggle: () => void } & React.Attributes) => {
    const tag = LEVEL_TAG[entry.level];
    const dim = entry.level === "debug";
    const row = (
        <row spacing={8}>
            <text size={10} width={40} font="monospace" color={tag.color}>{tag.label}</text>
            <col width="fill" spacing={2}>
                <row spacing={6}>
                    {entry.source ? (
                        <view padding={[0, 5]} style={{ background: CORE_SOURCES.has(entry.source) ? "background.weak" : "#2c3a52", borderRadius: 8 }}>
                            <text size={11} color={CORE_SOURCES.has(entry.source) ? MUTED : "#9cc3ff"}>{entry.source}</text>
                        </view>
                    ) : null}
                    <text size={12} font="monospace" width="fill" wrapping="wordOrGlyph" color={dim ? MUTED : "background.text"}>{entry.message}</text>
                </row>
                {entry.trace.length ? (
                    expanded ? (
                        <col spacing={1}>
                            {entry.trace.map((line, i) => <text key={i} size={11} font="monospace" wrapping="wordOrGlyph" color={MUTED}>{line}</text>)}
                            <button width="shrink" style="text" padding={0} onPress={onToggle}>
                                <text size={11} color="primary">Hide stack trace</text>
                            </button>
                        </col>
                    ) : (
                        <button width="shrink" style="text" padding={0} onPress={onToggle}>
                            <text size={11} color="primary">{`Show stack trace (${entry.trace.length} lines)`}</text>
                        </button>
                    )
                ) : null}
            </col>
        </row>
    );

    return entry.level === "error"
        ? <view width="fill" padding={[2, 12, 2, 12]} style={{ background: "#c3423f1f" }}>{row}</view>
        : <view width="fill" padding={[2, 12, 2, 12]}>{row}</view>;
};

export const Logs = () => {
    const app = useApp();
    const queryClient = useQueryClient();
    const [chosenProfile, setChosenProfile] = useState<string | null>(null);
    const [follow, setFollow] = useState(true);
    const [query, setQuery] = useState("");
    const [levels, setLevels] = useState<Set<Level>>(() => new Set(["error", "warning", "info"]));
    const [source, setSource] = useState("");
    const [expanded, setExpanded] = useState<Set<number>>(() => new Set());

    const targetId = chosenProfile ?? app.activeProfile;
    const options = useProfileOptions();
    const { data: profile } = useProfile(targetId);

    const log = useQuery({
        queryKey: ["log", profile?.dir],
        enabled: !!profile,
        refetchInterval: follow ? 1000 : false,
        queryFn: async ({ queryKey }): Promise<LogState> => {
            const prev = queryClient.getQueryData<LogState>(queryKey);
            const result = await invoke<LogFile | { error: string }>("logs::read", {
                profile_dir: profile!.dir,
                size: prev?.size ?? 0,
                modified: prev?.modified ?? 0,
            });
            if ("error" in result) throw new Error(result.error);

            const { content, ...meta } = result;
            if (content === null && result.exists && prev) return { ...prev, ...meta };
            return { ...meta, entries: content ? parseLog(content) : [] };
        },
    });

    const entries = log.data?.entries ?? [];
    const counts = useMemo(() => {
        const out: Record<Level, number> = { error: 0, warning: 0, info: 0, debug: 0 };
        for (const e of entries) out[e.level]++;
        return out;
    }, [entries]);

    const sources = useMemo(() => {
        const all = [...new Set(entries.map(e => e.source).filter(Boolean))];
        all.sort((a, b) => Number(CORE_SOURCES.has(a)) - Number(CORE_SOURCES.has(b)) || a.localeCompare(b));
        return [{ value: "", label: "All sources" }, ...all.map(s => ({ value: s, label: s }))];
    }, [entries]);

    const q = query.trim().toLowerCase();
    const matches = (e: Entry) => !q || e.message.toLowerCase().includes(q) || e.source.toLowerCase().includes(q) || e.trace.some(t => t.toLowerCase().includes(q));
    const visible = entries
        .map((entry, index) => ({ entry, index }))
        .filter(({ entry }) =>
            levels.has(entry.level)
            && (!source || entry.source === source)
            && (!app.modOnlyLogLines || !CORE_SOURCES.has(entry.source))
            && matches(entry));
    const shown = visible.slice(-MAX_SHOWN);

    const toggleLevel = (level: Level) => setLevels(prev => {
        const next = new Set(prev);
        next.has(level) ? next.delete(level) : next.add(level);
        return next;
    });
    const toggleTrace = (index: number) => setExpanded(prev => {
        const next = new Set(prev);
        next.has(index) ? next.delete(index) : next.add(index);
        return next;
    });
    const clearFilters = () => {
        setQuery("");
        setSource("");
        setLevels(new Set(["error", "warning", "info", "debug"]));
        app.updateSettings({ modOnlyLogLines: false });
    };

    const exportLog = useMutation({
        mutationFn: async () => {
            const result = await invoke<{ path?: string | null, error?: string }>("logs::export", { profile_dir: profile!.dir });
            if (result.error) throw new Error(result.error);
            return result.path ?? null;
        },
    });

    const launch = useMutation({
        mutationFn: async () => {
            const result = await invoke<{ error?: string }>("launch-game", { profile_dir: profile!.dir, launch_args: profile!.launch_args });
            if (result.error) throw new Error(result.error);
        },
    });

    const file = log.data;

    return (
        <col height="fill">
            <row spacing={8} padding={[10, 16, 8, 12]} alignY="center">
                <text size={15} font={{ weight: "semibold" }}>Logs</text>
                {options.length ? (
                    <select width="fill" options={options} placeholder="Select profile" value={targetId ?? undefined} onSelect={(ev) => setChosenProfile(ev.value)} />
                ) : <space width="fill" />}
                <input type="switch" label="Follow" checked={follow} onChange={(ev) => setFollow(ev.value)} textSize={12} />
            </row>

            <col spacing={6} padding={[0, 12, 8, 12]}>
                <row spacing={8} alignY="center">
                    <input type="text" size={13} padding={[6, 8]} placeholder="Search log" value={query} onChange={(ev) => setQuery(ev.value)} />
                    <input type="switch" label="Mods only" checked={app.modOnlyLogLines} onChange={(ev) => app.updateSettings({ modOnlyLogLines: ev.value })} textSize={12} />
                </row>
                <row spacing={4} alignY="center">
                    {LEVELS.map(({ level, label, palette }) => {
                        const on = levels.has(level);
                        return (
                            <button key={level} width="shrink" padding={[1, 8]} onPress={() => toggleLevel(level)} style={{
                                background: on ? (level === "error" ? "#c3423f40" : level === "warning" ? "#b8892c40" : "background.weak") : "background",
                                borderColor: on ? palette : "background.strong",
                                borderWidth: 1,
                                borderRadius: 10,
                                color: on ? "background.text" : MUTED,
                            }}>
                                <text size={12}>{`${label} ${counts[level]}`}</text>
                            </button>
                        );
                    })}
                    <space width="fill" />
                    <select options={sources} value={source} onSelect={(ev) => setSource(ev.value)} />
                </row>
            </col>
            <hr />

            {!options.length ? (
                <Center spacing={8}>
                    <text color={MUTED}>Create a profile on the Profiles page to see its log.</text>
                </Center>
            ) : !profile ? (
                <Center><text color={MUTED}>Pick a profile to see its log.</text></Center>
            ) : log.isLoading ? (
                <Center><text color={MUTED}>Reading log…</text></Center>
            ) : log.isError ? (
                <Center spacing={8}>
                    <svg height={40} width={40} color="danger" src={TriangleAlert} />
                    <text>Couldn't read the log.</text>
                    <text size={12} color={MUTED}>{log.error.message}</text>
                </Center>
            ) : !file?.exists ? (
                <Center spacing={8}>
                    <svg height={40} width={40} color={MUTED} src={ScrollText} />
                    <text>No log for this profile yet.</text>
                    <text size={12} color={MUTED} center>BepInEx writes LogOutput.log the first time the game launches with this profile.</text>
                    <button width="shrink" disabled={launch.isPending} onPress={() => launch.mutate()}>
                        <text size={13}>{launch.isPending ? "Launching…" : `Launch ${profile.name}`}</text>
                    </button>
                    {launch.error ? <text size={12} color="danger">{launch.error.message}</text> : null}
                </Center>
            ) : !shown.length ? (
                <Center spacing={8}>
                    <text>{entries.length ? "No lines match." : "The log is empty."}</text>
                    {entries.length ? (
                        <button width="shrink" style="secondary" onPress={clearFilters}>
                            <text size={13}>Clear filters</text>
                        </button>
                    ) : null}
                </Center>
            ) : (
                <scroll height="fill" spacing={4} anchorBottom={follow}>
                    <col padding={[4, 0, 8, 0]}>
                        {visible.length > shown.length ? (
                            <text size={12} color={MUTED} center width="fill">{`Showing the newest ${shown.length} of ${visible.length} lines`}</text>
                        ) : null}
                        {shown.map(({ entry, index }) => (
                            <LogEntry
                                key={index}
                                entry={entry}
                                expanded={expanded.has(index) || (!!q && entry.trace.some(t => t.toLowerCase().includes(q)))}
                                onToggle={() => toggleTrace(index)}
                            />
                        ))}
                    </col>
                </scroll>
            )}

            <hr />
            <row spacing={6} padding={[6, 16, 6, 12]} alignY="center">
                <col width="fill" spacing={1}>
                    <text size={11} font="monospace" color={MUTED} wrapping="glyph">{file?.path ?? ""}</text>
                    <text size={12} color={exportLog.error ? "danger" : MUTED}>{
                        exportLog.error ? `Export failed: ${exportLog.error.message}`
                            : exportLog.data ? `Saved to ${exportLog.data}`
                                : file?.exists ? `${entries.length} entries · ${kb(file.size)} · updated ${ago(file.modified)}`
                                    : file ? "File not found" : ""
                    }</text>
                </col>
                <button width="shrink" style="subtle" disabled={!file} onPress={() => file && void invoke("open-path", { path: file.path }).catch(console.error)}>
                    <svg height={15} width={15} color="background.text" src={FolderIcon} />
                </button>
                <button width="shrink" style="secondary" disabled={!file?.exists || exportLog.isPending} onPress={() => exportLog.mutate()}>
                    <row spacing={4} alignY="center">
                        <svg height={12} width={12} color="secondary.text" src={Download} />
                        <text size={13}>Export…</text>
                    </row>
                </button>
            </row>
        </col>
    );
}
