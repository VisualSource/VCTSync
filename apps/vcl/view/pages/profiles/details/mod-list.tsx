import { useState } from "react";
import { useModList, useProfileCollection } from "../query.js";
import Trash from "@icon/trash.svg" with {type: "svg" };
import PackagePlus from "@icon/package-plus.svg" with {type: "svg"};
import TriangleAlert from "@icon/triangle-alert.svg" with { type: "svg" };
import { useProfileState } from "../state.js";

export const ModsList = () => {
    const profileId = useProfileState(state => state.detailId);
    const toggleCatlog = useProfileState(state => state.toggleCatlog);

    const [query, setQuery] = useState("");
    const { data: mods } = useModList(profileId, query);
    const collection = useProfileCollection();

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
                <button width="shrink" onPress={toggleCatlog}>
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
                            <view key={e.id} style={{ background: "background.weaker", borderRadius: 4 }}>
                                <row padding={6} spacing={6} alignY="center">
                                    <view height={30} width={30}>
                                        <svg src={Trash} />
                                    </view>
                                    <col>
                                        <text color="primary.base.text" font={{ weight: "semibold" }} size={14}>{e.name}</text>
                                        <text size={12}>{`${e.author} · ${e.version}`}</text>
                                    </col>

                                    <space width="fill" />

                                    <input type="switch" checked={e.enabled} onChange={(ev) => {
                                        collection.update(profileId, draft => {
                                            const mod = draft.mods.findIndex(mod => mod.id === e.id);
                                            if (mod !== -1 && draft.mods[mod]) draft.mods[mod].enabled = ev.value;
                                        })
                                    }} />
                                    <button width={30} style="subtle" height={30} padding={6} onPress={() => {
                                        collection.update(profileId, draft => {
                                            draft.mods = draft.mods.filter(mod => mod.id !== e.id);
                                        });
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