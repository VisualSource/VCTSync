import { useState } from "react";
import { invoke } from "react-iced-native";
import { useModList, useProfile, useProfileCollection } from "../query.js";
import { message } from "../../../platform/dialog.js";
import Trash from "@icon/trash.svg" with {type: "svg" };
import PackagePlus from "@icon/package-plus.svg" with {type: "svg"};
import TriangleAlert from "@icon/triangle-alert.svg" with { type: "svg" };
import { useProfileState } from "../state.js";
import { ProfileIcon } from "../profile-icon.js";
import { pretty } from "../catalog/requets.js";

export const ModsList = () => {
    const profileId = useProfileState(state => state.detailId);
    const toggleCatlog = useProfileState(state => state.toggleCatlog);

    const [query, setQuery] = useState("");
    const { data: mods } = useModList(profileId, query);
    const collection = useProfileCollection();
    const { data: profile } = useProfile(profileId);

    if (!mods || !profileId) return (<col />);

    return (
        <>
            <row spacing={4}>
                <input
                    size={14}
                    padding={[8, 8]}
                    type="text"
                    value={query}
                    onChange={(ev) => setQuery(ev.value)}
                    placeholder="Search mods"
                />
                <button width="shrink" onPress={() => toggleCatlog(true)}>
                    <row alignX="center" alignY="center" spacing={2}>
                        <svg height={16} width={16} color="background.text" src={PackagePlus} />
                        <text size={16}>Add Mod</text>
                    </row>
                </button>
            </row>

            <space height={10} />

            {mods.length === 0 ? (
                <col height="fill" alignX="center" alignY="center">
                    <view center="fill">
                        <col spacing={4} alignX="center">
                            <svg height={48} width={48} color="background.text" src={TriangleAlert} />
                            <text>{query.length ? `No mods with name of '${query}'` : "No mods"}</text>
                        </col>
                    </view>
                </col>
            ) : (
                <scroll spacing={4}>
                    <col spacing={6}>
                        {mods.map(e => (
                            <view key={e.id} style={{ background: "background.weaker", opacity: e.enabled ? 1 : 0.45, borderRadius: 4 }}>
                                <row padding={6} spacing={6} alignY="center">
                                    {e.icon ? <img src={e.icon} width={32} height={32} borderRadius={4} /> : <ProfileIcon type="box" />}
                                    <col>
                                        <text color="primary.base.text" font={{ weight: "semibold" }} size={14}>{pretty(e.name)}</text>
                                        <text size={12}>{`${e.author} · v${e.version}`}</text>
                                    </col>

                                    <space width="fill" />

                                    <input type="switch" checked={e.enabled} onChange={(ev) => {
                                        if (!profile) return;
                                        // Rename the files first so the record never claims a state the disk isn't in.
                                        invoke<{ error?: string }>("mods::set-enabled", { profile_dir: profile.dir, full_name: `${e.author}-${e.name}`, enabled: ev.value })
                                            .then(result => {
                                                if (result.error) throw new Error(result.error);
                                                collection.update(profileId, draft => {
                                                    const mod = draft.mods.find(mod => mod.id === e.id);
                                                    if (mod) mod.enabled = ev.value;
                                                });
                                            })
                                            .catch((err: Error) => message(err.message, { title: `Couldn't ${ev.value ? "enable" : "disable"} ${pretty(e.name)}`, buttons: "Ok", kind: "error" }))
                                            .catch(console.error);
                                    }} />
                                    <button width={30} style="subtle" height={30} padding={6} onPress={() => {
                                        if (!profile) return;
                                        // Files first: if removal fails the mod stays listed, matching what's on disk.
                                        invoke<{ error?: string }>("mods::uninstall", { profile_dir: profile.dir, full_name: `${e.author}-${e.name}` })
                                            .then(result => {
                                                if (result.error) throw new Error(result.error);
                                                collection.update(profileId, draft => {
                                                    draft.mods = draft.mods.filter(mod => mod.id !== e.id);
                                                });
                                            })
                                            .catch((err: Error) => message(err.message, { title: `Couldn't remove ${pretty(e.name)}`, buttons: "Ok", kind: "error" }))
                                            .catch(console.error);
                                    }}>
                                        <svg src={Trash} color="background.text" />
                                    </button>
                                </row>
                            </view>
                        ))}
                    </col>
                </scroll>)}
        </>
    );
}