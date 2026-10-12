import { useQuery } from "@tanstack/react-query";
import LayersPlus from "../../assets/icons/layers-plus.svg" with { type: "svg" };
import Settings from "../../assets/icons/settings.svg" with { type: "svg" };
import Play from "../../assets/icons/square-library.svg" with { type: "svg" };
import { invoke } from "react-iced-native";
import { useMemo } from "react";
import { useApp } from "../../state.js";

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
    const activeProfile = useApp(state => state.activeProfile);
    const setActiveProfile = useApp(state => state.setActiveProfile);
    const { isLoading, isError, error, data } = useQuery({
        queryKey: ["profiles"],
        queryFn: async () => {
            return [] as Profile[];
        }
    });

    const options = useMemo(() => data?.map(e => ({ label: e.name, value: e.id })) ?? [], []);

    return (
        <col>
            <row>
                <row spacing={2}>
                    <text>Active profile</text>
                    <select options={options} onSelect={(ev) => setActiveProfile(ev.value)} value={activeProfile ?? undefined} />
                </row>
                <space width="fill" />
                <button onPress={() => {
                    void invoke("launch-game", {}).catch((err) => {
                        console.error(err);
                    })
                }}>
                    <row>
                        <svg src={Play} />
                        <text>Launch</text>
                    </row>
                </button>
            </row>
            <hr />
            <space height={10} />
            <View activeProfile={activeProfile} isError={isError} isLoading={isLoading} error={error} data={data} />
        </col>
    )
}

const View = ({ isLoading, isError, error, data, activeProfile }: { activeProfile?: string | null, isLoading: boolean; isError: boolean; error: Error | null, data: Profile[] | undefined }) => {
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
                        <svg src={LayersPlus} />
                    </row>
                </button>
            </col>
        </view>
    );

    return (
        <col>
            <row>
                <text>PROFILES</text>
                <space width="fill" />
                <button>
                    <row>
                        <svg src={LayersPlus} />
                        <text>New Profile</text>
                    </row>
                </button>
            </row>
            <scroll>
                <col>
                    {data.map(profile => (
                        <view key={profile.id}>
                            <row>
                                <view>
                                    <row />
                                </view>
                                <col>
                                    <text>{profile.name}</text>
                                    <text>{`Mods ${profile.mods.length}/${profile.mods.length}`}</text>
                                </col>
                                <space width="fill" />

                                {profile.id === activeProfile ? (
                                    <view>
                                        <text>Active</text>
                                    </view>
                                ) : null}

                                <button>
                                    <svg src={Settings} />
                                </button>
                            </row>
                        </view>
                    ))}
                </col>
            </scroll>
        </col>
    );
}