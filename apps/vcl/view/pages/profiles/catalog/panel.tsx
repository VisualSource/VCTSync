import BackArrow from "@icon/arrow-left.svg";
import Archive from "@icon/file-archive.svg";
import Refresh from "@icon/refresh-cw.svg";
import { useState } from "react";
import { useProfileState } from "../state.js";
import { useQuery } from "@tanstack/react-query";

const SELECT_OPTIONS = [{
    label: "Popular",
    value: "popular"
}]

export const Catalog = () => {
    const [filterBy, setFilterBy] = useState("popular");
    const [query, setQuery] = useState("");

    const toggle = useProfileState(state => state.toggleCatlog);

    const { data, isLoading, isError, error } = useQuery({
        queryKey: ["voidcrew-mod-list"],
        queryFn: async () => { }
    })

    return (
        <opaque>
            <view style={{ background: "background" }} height="fill">
                <col spacing={6} padding={6}>
                    <row spacing={6} alignY="center">
                        <button width="shrink" style="subtle" onPress={() => toggle()}>
                            <svg height={15} width={15} color="background.text" src={BackArrow} />
                        </button>
                        <col>
                            <text size={14} font={{ weight: "semibold" }}>Add Mods</text>
                            <text size={12} color="background.strongest.text">Back</text>
                        </col>

                        <space width="fill" />

                        <button onPress={() => { }} style="secondary" width="shrink">
                            <row spacing={6} alignX="center" alignY="center">
                                <svg height={15} width={15} color="secondary.text" src={Archive} />
                                <text size={14}>From .zip</text>
                            </row>
                        </button>
                    </row>

                    <row spacing={6}>
                        <input placeholder="Search..." type="text" onChange={(ev) => setQuery(ev.value)} value={query} />
                        <select onSelect={(ev) => setFilterBy(ev.value)} options={SELECT_OPTIONS} value={filterBy} />
                    </row>

                    {
                        isLoading ? (
                            <col><text>Loading...</text></col>
                        ) : isError ? (
                            <col><text>{`${error.message}`}</text></col>
                        ) : (
                            <scroll height="fill" spacing={2}>
                                <col>

                                </col>
                            </scroll>
                        )
                    }

                    <hr />
                    <row alignX="center" alignY="center" padding={[0, 4, 4]}>
                        <text size={12}>synced 3 min ago</text>
                        <space width="fill" />
                        <button width="shrink" style="subtle">
                            <svg color="background.text" width={15} height={15} src={Refresh} />
                        </button>
                    </row>
                </col>
            </view>
        </opaque>
    );
}