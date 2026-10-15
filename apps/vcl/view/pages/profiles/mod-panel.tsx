import { useState } from "react";

import XIcon from "@icon/x.svg" with {type: "svg" };
import SlidersHorizontal from "@icon/sliders-horizontal.svg" with { type: "svg" };
import Boxes from "@icon/boxes.svg" with { type: "svg" };

import { ProfileIcon, } from "./profile-icon.js";
import { useProfile } from "./query.js";
import { SettingsTab } from "./mod-panel-settings.js";
import { ModsList } from "./mod-panel-mod-list.js";

type Props = {
    onClose: () => void;
    onDelete: (id: string) => void;
    profileId: string | null,
}

export const ModPanel = ({ onClose, onDelete, profileId }: Props) => {
    const [tab, setTab] = useState("settings");

    const { data, isLoading, isError, error } = useProfile(profileId);

    if (!profileId) return null;

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
                            <row alignX="center" alignY="center">
                                <row spacing={6} alignX="center" alignY="center">
                                    <ProfileIcon type={data?.icon ?? "box"} />
                                    <text size={14} font={{ weight: "semibold" }}>{data!?.name}</text>
                                </row>
                                <space width="fill" />
                                <button width="shrink" style="subtle" onPress={onClose}>
                                    <svg height={18} width={18} src={XIcon} style="currentColor" />
                                </button>
                            </row>
                            <space height={8} />
                            <row spacing={4}>
                                <button style={tab === "settings" ? "primary" : "secondary"} onPress={() => setTab("settings")}>
                                    <row spacing={4}>
                                        <svg height={15} width={15} src={SlidersHorizontal} style="currentColor" />
                                        <text size={14}>Profile Settings</text>
                                    </row>
                                </button>
                                <button style={tab === "mods" ? "primary" : "secondary"} onPress={() => setTab("mods")}>
                                    <row spacing={4}>
                                        <svg height={15} width={15} src={Boxes} style="currentColor" />
                                        <text size={14}>Mods</text>
                                    </row>
                                </button>
                            </row>
                            <space height={18} />

                            {tab === "settings" ? <SettingsTab onDelete={onDelete} data={data!} /> : <ModsList profileId={profileId} />}
                        </col>
                    )
                }
            </view>
        </opaque>
    );
}