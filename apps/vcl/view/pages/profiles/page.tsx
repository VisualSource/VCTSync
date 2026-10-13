import { useQuery } from "@tanstack/react-query";
import LayersPlus from "../../assets/icons/layers-plus.svg" with { type: "svg" };
import Settings from "../../assets/icons/settings.svg" with { type: "svg" };
import Play from "../../assets/icons/play.svg" with { type: "svg" };
import { AnimActivity, invoke } from "react-iced-native";
import { useApp } from "../../state.js";
import { ModPanel } from "./mod-panel.js";
import { useState } from "react";

type Profile = {
    name: string;
    id: string;
    mods: {
        name: string;
        icon: string;
        id: string;
    }[]
}

export const Profiles = () => {
    const [showDetails, setShowDetails] = useState(false);
    const activeProfile = useApp(state => state.activeProfile);
    const setActiveProfile = useApp(state => state.setActiveProfile);
    const { isLoading, isError, error, data } = useQuery({
        queryKey: ["profiles"],
        queryFn: async () => {
            return [{ name: "Example", id: "test example", mods: [] }] as Profile[];
        }
    });

    const options = data?.map(e => ({ label: e.name, value: e.id })) ?? [];

    return (
        <stack>
            <col height="fill">
                <row padding={6}>
                    <row spacing={2} alignY="center">
                        <select options={options} placeholder="Select profile" onSelect={(ev) => setActiveProfile(ev.value)} value={activeProfile ?? undefined} />
                    </row>
                    <space width="fill" />
                    <button onPress={() => {
                        void invoke("launch-game", {}).catch((err) => {
                            console.error(err);
                        })
                    }}>
                        <row alignX="center" alignY="center">
                            <svg height={15} width={15} style="currentColor" src={Play} />
                            <text>Launch</text>
                        </row>
                    </button>
                </row>
                <hr />
                <space height={10} />
                <View toggleShowDetails={() => setShowDetails(e => !e)} activeProfile={activeProfile} isError={isError} isLoading={isLoading} error={error} data={data} />
            </col>

            <AnimActivity active={showDetails} from={{ x: "100%" }} to={{ x: 0 }} duration={220}>
                <ModPanel onClose={() => setShowDetails(false)} />
            </AnimActivity>
        </stack>
    )
}

const View = ({ isLoading, isError, error, data, activeProfile, toggleShowDetails }: { toggleShowDetails: () => void, activeProfile?: string | null, isLoading: boolean; isError: boolean; error: Error | null, data: Profile[] | undefined }) => {
    if (isLoading) return (
        <view height="fill" center="fill" width="fill">
            <col alignX="center" spacing={6}>
                <text>Loading</text>
            </col>
        </view>
    );


    if (isError) return (
        <view height="fill" center="fill" width="fill">
            <col alignX="center" spacing={6}>
                <text>{error?.message ?? String(error)}</text>
            </col>
        </view>
    );


    if (!data) return (
        <view height="fill" center="fill" width="fill">
            <col alignX="center" spacing={6}>
                <text>There was an issue loading profiles try again</text>
            </col>
        </view>
    );

    if (data.length === 0) return (
        <view height="fill" center="fill" width="fill">
            <col alignX="center" spacing={6}>
                <text center>No profiles</text>
                <button onPress={() => { }}>
                    <row width="shrink" spacing={6}>
                        <text>Create</text>
                        <svg style="currentColor" src={LayersPlus} />
                    </row>
                </button>
            </col>
        </view>
    );

    return (
        <col padding={[0, 4]} spacing={6}>
            <row alignX="center" alignY="center">
                <text size={14}>PROFILES</text>
                <space width="fill" />
                <button width="shrink" onPress={() => { }}>
                    <row alignY="center" spacing={4}>
                        <svg style="currentColor" height={15} width={15} src={LayersPlus} />
                        <text size={14}>New Profile</text>
                    </row>
                </button>
            </row>
            <scroll>
                <col>
                    {data.map(profile => (
                        <view key={profile.id} style={{ background: "background.weaker", borderRadius: 4 }}>
                            <row padding={6} spacing={6} alignY="center">
                                <view alignX="center" alignY="center" style={{ background: "#2f8a5b", borderRadius: 4 }} width={30} height={30}>
                                    <svg style="currentColor" height={15} width={15} src={LayersPlus} />
                                </view>
                                <col>
                                    <text size={14}>{profile.name}</text>
                                    <text size={12}>{`Mods ${profile.mods.length}/${profile.mods.length}`}</text>
                                </col>
                                <space width="fill" />

                                {profile.id === activeProfile ? (
                                    <view style={{ background: "primary", borderRadius: 9 }} padding={[1, 6]}>
                                        <text size={11}>Active</text>
                                    </view>
                                ) : null}

                                <button height={36} width={36} style="subtle" onPress={() => toggleShowDetails()}>
                                    <svg height={30} width={30} style="currentColor" src={Settings} />
                                </button>
                            </row>
                        </view>
                    ))}
                </col>
            </scroll>
        </col>
    );
}