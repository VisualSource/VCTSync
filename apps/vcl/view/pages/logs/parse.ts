export type Level = "error" | "warning" | "info" | "debug";

export type Entry = {
    level: Level;
    source: string;
    message: string;
    /** Lines without a `[Level : Source]` prefix that follow this entry (stack traces). */
    trace: string[];
};

/** BepInEx's own lines and forwarded Unity output; everything else is a mod. */
export const CORE_SOURCES = new Set(["BepInEx", "Unity Log"]);

const LINE = /^\[(\w+)\s*:\s*([^\]]*?)\s*\] ?(.*)$/;

const LEVELS: Record<string, Level> = {
    fatal: "error",
    error: "error",
    warning: "warning",
    message: "info",
    info: "info",
    debug: "debug",
};

/** Parse BepInEx's LogOutput.log: `[Level  :  Source] message`, continuation lines attach to the previous entry. */
export const parseLog = (text: string): Entry[] => {
    const entries: Entry[] = [];
    for (const line of text.split(/\r?\n/)) {
        if (!line.trim()) continue;

        const match = LINE.exec(line);
        // "[Photon] connected" on a trace line has no known level, so it stays a continuation.
        const level = match && LEVELS[match[1]!.toLowerCase()];
        if (match && level) {
            entries.push({ level, source: match[2]!, message: match[3]!, trace: [] });
        } else if (entries.length) {
            entries.at(-1)!.trace.push(line.trimEnd());
        } else {
            entries.push({ level: "info", source: "", message: line, trace: [] });
        }
    }
    return entries;
};
