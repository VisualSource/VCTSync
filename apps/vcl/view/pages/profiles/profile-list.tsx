import LayersPlus from "../../assets/icons/layers-plus.svg" with { type: "svg" };
import Settings from "../../assets/icons/settings.svg" with { type: "svg" };
import { ProfileIcon } from "./profile-icon.js";
import { useListProfiles } from "./query.js";
import { useApp } from "../../state.js";

type Props = {
    toggleShowDetails: (id: string) => void,
}

export const ProfileList = ({ toggleShowDetails }: Props) => {
    const activeProfile = useApp(state => state.activeProfile);
    const { data, error, isError, isLoading } = useListProfiles();

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
                <text>{(error as Error)?.message ?? String(error)}</text>
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
                                <ProfileIcon type={profile.icon} />
                                <col>
                                    <text size={14}>{profile.name}</text>
                                    <text size={12} color="#9a9ea6">{`Mods ${profile.mods.length}/${profile.mods.length}`}</text>
                                </col>
                                <space width="fill" />

                                {profile.id === activeProfile ? (
                                    <view style={{ background: "primary", borderRadius: 9 }} padding={[1, 6]}>
                                        <text size={11}>Active</text>
                                    </view>
                                ) : null}

                                <button height={36} width={36} style="subtle" onPress={() => toggleShowDetails(profile.id)}>
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

