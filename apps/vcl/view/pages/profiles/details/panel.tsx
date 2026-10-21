import { useState } from "react";

import XIcon from "@icon/x.svg" with {type: "svg" };
import SlidersHorizontal from "@icon/sliders-horizontal.svg" with { type: "svg" };
import Boxes from "@icon/boxes.svg" with { type: "svg" };

import { ProfileIcon, } from "../profile-icon.js";
import { useProfile } from "../query.js";
import { SettingsTab } from "./settings.js";
import { ModsList } from "./mod-list.js";
import { useProfileState } from "../state.js";

const btnTabStyle: Iced.Elements.ButtonStyle = {
    color: "primary.base.text",
    disabledOpacity: 0.5
}

const btnTabInactiveStyle: Iced.Elements.ButtonStyle = {
    color: "primary.base.text",
    opacity: 0.5
}


export const ModPanel = () => {
    const [tab, setTab] = useState("mods");
    const profileId = useProfileState(state => state.detailId);
    const onClose = useProfileState(state => state.toggleDetails);

    const { data, isLoading, isError, error } = useProfile(profileId);

    if (!profileId) return null;

    const enabledMods = data?.mods.reduce((prev, mod) => mod.enabled ? prev + 1 : prev, 0);

    return (
        <opaque>
            <view style={{ background: "background" }} height="fill">
                {
                    isLoading ? (
                        <col></col>
                    ) : isError ? (
                        <col alignX="center" alignY="center">
                            <text>{(error as Error)?.message ?? String(error)}</text>
                        </col>
                    ) : (
                        <col padding={4}>
                            <row alignX="center" alignY="center" >
                                <row spacing={6} alignX="center" alignY="center">
                                    <ProfileIcon type={data?.icon ?? "box"} />
                                    <text size={14} font={{ weight: "semibold" }}>{data?.name ?? ""}</text>
                                </row>
                                <space width="fill" />
                                <button width="shrink" style="subtle" onPress={onClose}>
                                    <svg height={18} width={18} src={XIcon} color="background.text" />
                                </button>
                            </row>
                            <space height={8} />
                            <row>
                                <col width="shrink">
                                    <button style={tab !== "mods" ? btnTabInactiveStyle : btnTabStyle} onPress={() => setTab("mods")}>
                                        <row spacing={4}>
                                            <svg height={15} width={15} src={Boxes} color="background.text" opacity={tab !== "mods" ? 0.5 : 1} />
                                            <text size={14}>{data?.mods.length !== 0 ? `Mods ${enabledMods}/${data?.mods.length}` : "Mods"}</text>
                                        </row>
                                    </button>
                                    {
                                        tab === "mods" ? (
                                            <view width="fill" height={2} style={{ background: "primary" }}>
                                                <space />
                                            </view>
                                        ) : null
                                    }
                                </col>
                                <col width="shrink">
                                    <button style={tab !== "settings" ? btnTabInactiveStyle : btnTabStyle} onPress={() => setTab("settings")}>
                                        <row spacing={4}>
                                            <svg height={15} width={15} src={SlidersHorizontal} color="background.text" opacity={tab !== "settings" ? 0.5 : 1} />
                                            <text size={14}>Settings</text>
                                        </row>
                                    </button>
                                    {
                                        tab === "settings" ? (
                                            <view width="fill" height={2} style={{ background: "primary" }}>
                                                <space />
                                            </view>
                                        ) : null
                                    }
                                </col>
                            </row>
                            <hr />
                            <space height={18} />

                            {tab === "settings" ? <SettingsTab data={data!} /> : <ModsList />}
                        </col>
                    )
                }
            </view>
        </opaque>
    );
}