import BackArrow from "@icon/arrow-left.svg";
import Archive from "@icon/file-archive.svg";
import Refresh from "@icon/refresh-cw.svg";
import Download from "@icon/download.svg";
import ThumbsUp from "@icon/thumbs-up.svg";
import ChevronRight from "@icon/chevron-right.svg";
import WifiOff from "@icon/wifi-off.svg";
import { useState } from "react";
import { useProfileState } from "../state.js";
import { useProfile } from "../query.js";
import { formatCount, pickZip, pretty, statusOf, totalDownloads, useCatalog, useInstalled, type InstallStatus } from "./requets.js";
import { ProfileIcon } from "../profile-icon.js";
import type { CatalogMod } from "./types.js";

type Sort = "popular" | "rating" | "updated" | "name";

const SELECT_OPTIONS: { label: string, value: Sort }[] = [
    { label: "Popular", value: "popular" },
    { label: "Top rated", value: "rating" },
    { label: "Updated", value: "updated" },
    { label: "Name", value: "name" },
];

const SORTERS: Record<Sort, (a: CatalogMod, b: CatalogMod) => number> = {
    popular: (a, b) => totalDownloads(b) - totalDownloads(a),
    rating: (a, b) => b.rating_score - a.rating_score,
    updated: (a, b) => b.date_updated.localeCompare(a.date_updated),
    name: (a, b) => a.name.localeCompare(b.name),
};

const CHIPS = {
    installed: { label: "Installed", palette: "success" },
    outdated: { label: "Update", palette: "warning" },
    missing: { label: "Will install", palette: "warning" },
    notfound: { label: "Not found", palette: "danger" },
} as const;

export const StatusChip = ({ status }: { status: InstallStatus | keyof typeof CHIPS }) => {
    if (status === "none") return null;
    const { label, palette } = CHIPS[status];

    return (
        <view padding={[1, 6]} style={{ background: palette, borderRadius: 9 }}>
            <text size={11} color={`${palette}.text`}>{label}</text>
        </view>
    );
}

export const Catalog = () => {
    const [sort, setSort] = useState<Sort>("popular");
    const [query, setQuery] = useState("");

    const toggle = useProfileState(state => state.toggleCatlog);
    const openModDetails = useProfileState(state => state.openModDetails);
    const openZipInstall = useProfileState(state => state.openZipInstall);
    const profileId = useProfileState(state => state.detailId);
    const { data: profile } = useProfile(profileId);
    const installed = useInstalled();

    const { data, isLoading, isError, refetch, dataUpdatedAt } = useCatalog();

    const q = query.trim().toLowerCase();
    const list = data?.mods
        .filter(mod => !q || mod.name.toLowerCase().includes(q) || mod.owner.toLowerCase().includes(q) || mod.versions[0]?.description.toLowerCase().includes(q))
        .sort(SORTERS[sort]);

    const syncedMin = Math.round((Date.now() - dataUpdatedAt) / 60_000);

    return (
        <opaque>
            <view style={{ background: "background" }} height="fill">
                <col spacing={6} padding={6}>
                    <row spacing={6} alignY="center">
                        <button width="shrink" style="subtle" onPress={() => toggle(false)}>
                            <svg height={15} width={15} color="background.text" src={BackArrow} />
                        </button>
                        <col>
                            <text size={14} font={{ weight: "semibold" }}>Add mods</text>
                            {profile ? <text size={12} color="background.strongest.text">{`to ${profile.name}`}</text> : null}
                        </col>

                        <space width="fill" />

                        <button style="secondary" width="shrink" onPress={() => {
                            pickZip().then(result => {
                                if (result.path) openZipInstall(result.path);
                            }).catch(err => console.error(err));
                        }}>
                            <row spacing={6} alignX="center" alignY="center">
                                <svg height={15} width={15} color="secondary.text" src={Archive} />
                                <text size={14}>From .zip</text>
                            </row>
                        </button>
                    </row>

                    <row spacing={6}>
                        <input placeholder="Search Thunderstore" type="text" onChange={(ev) => setQuery(ev.value)} value={query} />
                        <select onSelect={(ev) => setSort(ev.value as Sort)} options={SELECT_OPTIONS} value={sort} />
                    </row>

                    {
                        isLoading ? (
                            <col height="fill" alignX="center" alignY="center">
                                <text color="background.strongest.text">Fetching package list from thunderstore.io…</text>
                            </col>
                        ) : isError || !list ? (
                            <col height="fill" alignX="center" alignY="center" spacing={8}>
                                <svg height={40} width={40} color="background.text" src={WifiOff} />
                                <text>Couldn't reach Thunderstore.</text>
                                <text size={12} color="background.strongest.text">You can still add a mod with From .zip.</text>
                                <button style="secondary" width="shrink" onPress={() => refetch()}>
                                    <row spacing={6} alignY="center">
                                        <svg height={15} width={15} color="secondary.text" src={Refresh} />
                                        <text size={14}>Retry</text>
                                    </row>
                                </button>
                            </col>
                        ) : list.length === 0 ? (
                            <col height="fill" alignX="center" alignY="center">
                                <text color="background.strongest.text">{`No mods match "${query}".`}</text>
                            </col>
                        ) : (
                            <scroll height="fill" spacing={4}>
                                <col spacing={4}>
                                    {list.map(mod => (
                                        <button key={mod.uuid4} padding={0} style={{ background: "background.weaker", hoverBackground: "background.weak", borderRadius: 4 }} onPress={() => openModDetails(mod.uuid4)}>
                                            <row spacing={8} padding={6} alignY="center">
                                                <ProfileIcon type="box" />
                                                <col width="fill">
                                                    <text font={{ weight: "semibold" }}>{pretty(mod.name)}</text>
                                                    <row spacing={10} alignY="center">
                                                        <text size={12} color="background.strongest.text">{mod.owner}</text>
                                                        <row spacing={3} alignY="center">
                                                            <svg height={12} width={12} color="background.strongest.text" src={Download} />
                                                            <text size={12} color="background.strongest.text">{formatCount(totalDownloads(mod))}</text>
                                                        </row>
                                                        <row spacing={3} alignY="center">
                                                            <svg height={12} width={12} color="background.strongest.text" src={ThumbsUp} />
                                                            <text size={12} color="background.strongest.text">{mod.rating_score.toString()}</text>
                                                        </row>
                                                    </row>
                                                </col>
                                                <StatusChip status={statusOf(installed, mod.full_name, mod.versions[0]?.version_number ?? "")} />
                                                <svg height={15} width={15} color="background.strongest.text" src={ChevronRight} />
                                            </row>
                                        </button>
                                    ))}
                                </col>
                            </scroll>
                        )
                    }

                    <hr />
                    <row alignX="center" alignY="center" padding={[0, 4, 4]}>
                        <text size={12} color="background.strongest.text">
                            {data && list ? `${list.length} of ${data.mods.length} · synced ${syncedMin < 1 ? "just now" : `${syncedMin} min ago`}` : isLoading ? "Syncing…" : "Last synced: never"}
                        </text>
                        <space width="fill" />
                        <button width="shrink" style="subtle" onPress={() => refetch()}>
                            <svg color="background.text" width={15} height={15} src={Refresh} />
                        </button>
                    </row>
                </col>
            </view>
        </opaque>
    );
}
