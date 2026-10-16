import LayersPlus from "@icon/layers-plus.svg" with { type: "svg" };
import Settings from "@icon/settings.svg" with { type: "svg" };
import { ICONS, ProfileIcon } from "./profile-icon.js";
import { useListProfiles, useProfileCollection } from "./query.js";
import { useApp } from "../../state.js";
import { nanoid } from "nanoid";
import { useProfileState } from "./state.js";

export const ProfileList = () => {
    const activeProfile = useApp(state => state.activeProfile);
    const { data, error, isError, isLoading } = useListProfiles();
    const collection = useProfileCollection();

    const openDetails = useProfileState(state => state.openDetails);

    const createProfile = () => {
        const time = Temporal.Now.plainDateTimeISO().toString();
        const id = nanoid();
        collection.insert({
            id,
            name: "Profile",
            mods: [],
            updated_at: time,
            created_at: time,
            launch_args: "",
            dir: `%APPDATA%/profiles/${id}`,
            icon: ICONS[Math.floor(Math.random() * ICONS.length)] ?? "box",
        });
    };

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
                <button onPress={createProfile}>
                    <row width="shrink" spacing={6}>
                        <text>Create</text>
                        <svg color="background.text" src={LayersPlus} />
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
                <button width="shrink" onPress={createProfile}>
                    <row alignY="center" spacing={4}>
                        <svg color="background.text" height={15} width={15} src={LayersPlus} />
                        <text size={14}>New Profile</text>
                    </row>
                </button>
            </row>
            <scroll spacing={2}>
                <col spacing={4}>
                    {data.map(profile => (
                        <view key={profile.id} style={{
                            borderWidth: profile.id === activeProfile ? 1 : undefined,
                            borderColor: profile.id === activeProfile ? "primary" : undefined,
                            background: "background.weaker",
                            borderRadius: 4
                        }}>
                            <row padding={6} spacing={6} alignY="center">
                                <ProfileIcon type={profile.icon} />
                                <col>
                                    <text color="primary.base.text" font={{ weight: "semibold" }} size={14}>{profile.name}</text>
                                    <text size={12}>{`Mods ${profile.mods.reduce((prev, mod) => mod.enabled ? prev + 1 : prev, 0)}/${profile.mods.length}`}</text>
                                </col>
                                <space width="fill" />

                                {profile.id === activeProfile ? (
                                    <view style={{ background: "primary", borderRadius: 9 }} padding={[1, 6]}>
                                        <text size={11}>Active</text>
                                    </view>
                                ) : null}

                                <button height={36} width={36} style="subtle" onPress={() => openDetails(profile.id)}>
                                    <svg height={30} width={30} color="background.text" src={Settings} />
                                </button>
                            </row>
                        </view>
                    ))}
                </col>
            </scroll>
            <space height={4} />
        </col>
    );
}

