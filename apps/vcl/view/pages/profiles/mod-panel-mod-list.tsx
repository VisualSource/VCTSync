import { useState } from "react";
import { useModList, useProfileCollection } from "./query.js";
import Trash from "@icon/trash.svg" with {type: "svg" };

export const ModsList = ({ profileId }: { profileId: string; }) => {
    const [query, setQuery] = useState("");
    const { data: mods } = useModList(profileId);
    const collection = useProfileCollection();

    if (!mods) return (<col />);

    return (
        <>
            <row>
                <input type="text" value={query} onChange={(ev) => setQuery(ev.value)} placeholder="Search mods" />
            </row>

            <space height={10} />

            {mods.length === 0 ? (
                <col height="fill" alignX="center" alignY="center">
                    <view>
                        <text>No mods</text>
                    </view>
                </col>
            ) : (
                <scroll>
                    <col spacing={6}>
                        {mods.map(e => (
                            <view key={e.id} style={{ background: "background.weaker", borderRadius: 4 }}>
                                <row padding={6} spacing={6} alignY="center">
                                    <view height={30} width={30}>
                                        <svg src={Trash} />
                                    </view>
                                    <col>
                                        <text size={14}>{e.name}</text>
                                        <text color="#9a9ea6" size={12}>{e.author}</text>
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
                                        <svg src={Trash} style="currentColor" />
                                    </button>
                                </row>
                            </view>
                        ))}
                    </col>
                </scroll>)}
        </>
    );
}