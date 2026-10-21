import BackArrow from "@icon/arrow-left.svg";
import Download from "@icon/download.svg";
import ThumbsUp from "@icon/thumbs-up.svg";
import Refresh from "@icon/refresh-cw.svg";
import Check from "@icon/check.svg";
import { useEffect } from "react";
import { useProfileState } from "../state.js";
import { ProfileIcon } from "../profile-icon.js";
import { StatusChip } from "./panel.js";
import { depKey, depVersion, formatCount, fromCatalog, missingDeps, pretty, statusOf, totalDownloads, useCatalog, useInstallMods, useInstalled } from "./requets.js";

const MUTED = "background.strongest.text";

export const DependencyList = ({ deps }: { deps: string[] }) => {
    const { data } = useCatalog();
    const installed = useInstalled();

    return (
        <col spacing={4}>
            <text size={12} color={MUTED}>{`DEPENDENCIES (${deps.length})`}</text>
            {deps.length === 0 ? <text size={12} color={MUTED}>None</text> : deps.map(dep => {
                const key = depKey(dep);
                return (
                    <view key={dep} style={{ background: "background.weaker", borderRadius: 3 }}>
                        <row spacing={8} padding={[4, 6]} alignY="center">
                            <text width="fill">{pretty(key.split("-")[1] ?? key)}</text>
                            <text size={12} color={MUTED}>{depVersion(dep)}</text>
                            <StatusChip status={installed.has(key) ? "installed" : data?.byName.has(key) ? "missing" : "notfound"} />
                        </row>
                    </view>
                );
            })}
        </col>
    );
}

export const ModDetailPanel = () => {
    const modId = useProfileState(state => state.pendingModId);
    const close = useProfileState(state => state.closeCatalogSubmenu);
    const { data } = useCatalog();
    const installed = useInstalled();
    const { mutate, isPending, error, step, reset } = useInstallMods();
    useEffect(() => reset(), [modId]);

    const mod = data?.mods.find(m => m.uuid4 === modId);
    const latest = mod?.versions[0];
    if (!mod || !latest || !data) return null;

    const status = statusOf(installed, mod.full_name, latest.version_number);
    const missing = missingDeps(latest.dependencies, installed, data.byName);
    const install = () => mutate([...missing, mod].map(fromCatalog));

    return (
        <opaque>
            <view style={{ background: "background" }} height="fill">
                <col spacing={8} padding={6}>
                    <row spacing={6} alignY="center">
                        <button width="shrink" style="subtle" onPress={close}>
                            <svg height={15} width={15} color="background.text" src={BackArrow} />
                        </button>
                        <text size={14} font={{ weight: "semibold" }}>{pretty(mod.name)}</text>
                    </row>

                    <scroll height="fill" spacing={4}>
                        <col spacing={8}>
                            <row spacing={10} alignY="center">
                                <ProfileIcon type="box" />
                                <col spacing={2}>
                                    <text size={16} font={{ weight: "semibold" }}>{pretty(mod.name)}</text>
                                    <text size={12} color={MUTED}>{`by ${mod.owner} Â· v${latest.version_number}`}</text>
                                    <row spacing={10} alignY="center">
                                        <row spacing={3} alignY="center">
                                            <svg height={12} width={12} color={MUTED} src={Download} />
                                            <text size={12} color={MUTED}>{formatCount(totalDownloads(mod))}</text>
                                        </row>
                                        <row spacing={3} alignY="center">
                                            <svg height={12} width={12} color={MUTED} src={ThumbsUp} />
                                            <text size={12} color={MUTED}>{mod.rating_score.toString()}</text>
                                        </row>
                                        <text size={12} color={MUTED}>{`updated ${mod.date_updated.slice(0, 10)}`}</text>
                                    </row>
                                </col>
                            </row>

                            <text>{latest.description}</text>

                            <DependencyList deps={latest.dependencies} />
                        </col>
                    </scroll>

                    <hr />
                    {error ? <text size={12} color="danger">{error.message}</text> : null}
                    <text size={12} color={MUTED}>{
                        isPending ? step
                            : status === "installed" ? "Installed in this profile."
                            : missing.length ? `Also installs ${missing.length} missing ${missing.length === 1 ? "dependency" : "dependencies"}.`
                                : "All dependencies present."
                    }</text>
                    {isPending ? (
                        <button disabled>
                            <text size={14} center>Installing…</text>
                        </button>
                    ) : status === "installed" ? (
                        <button style="success" disabled>
                            <view width="fill" alignX="center">
                                <row spacing={6} alignY="center">
                                    <svg height={15} width={15} color="success.text" src={Check} />
                                    <text size={14}>Installed</text>
                                </row>
                            </view>
                        </button>
                    ) : (
                        <button onPress={install}>
                            <view width="fill" alignX="center">
                                <row spacing={6} alignY="center">
                                    <svg height={15} width={15} color="primary.text" src={status === "outdated" ? Refresh : Download} />
                                    <text size={14}>{status === "outdated" ? `Update to v${latest.version_number}` : `Install${missing.length ? ` (+${missing.length})` : ""}`}</text>
                                </row>
                            </view>
                        </button>
                    )}
                </col>
            </view>
        </opaque>
    );
}
