
import Network from "../../assets/icons/network.svg" with { type: "svg" };
import FlaskConical from "../../assets/icons/flask-conical.svg" with { type: "svg" };

import type { GithubVersion, Version } from "./types.js";
import { InstalledVersion } from "./installed-version.js";
import { BuildGroup } from "./build-group.js";
import { useApp } from "../../state.js";

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

    return request.map(e => ({
        id: e.assets[0]?.node_id,
        version: e.tag_name,
        timestamp: formatDiff(Temporal.PlainDateTime.from(e.published_at.replace("Z", "")).until(now, { largestUnit: "days" })),
        content_type: "remote",
        source_url: e.assets[0]?.browser_download_url,
        digest: e.assets[0]?.digest,
        size: e.assets[0]?.size,
        branch: e.target_commitish
    } as Version));
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


export const Builds = () => {

    return (
        <col>
            <view>
                <text size={24} font={{ weight: "bold" }}>Versions</text>
            </view>
            <InstalledVersion />

            <space height={15} />

            <BuildGroup name="Remote" icon={Network} queryKey="builds::remote" queryFn={remoteQueryFn} />

            <space height={15} />

            <BuildGroup name="Local" icon={FlaskConical} queryKey="builds::local" queryFn={localQueryFn} />
        </col>
    );
}