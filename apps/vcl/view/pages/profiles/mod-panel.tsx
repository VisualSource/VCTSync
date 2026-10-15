import { useState } from "react";
import Trash from "@icon/trash.svg" with {type: "svg" };
import XIcon from "@icon/x.svg" with {type: "svg" };
import FolderInput from "@icon/folder-input.svg" with {type: "svg"};
import { IconButton, ProfileIcon, type Icon } from "./profile-icon.js";
import { confirmDialog, openDialog } from "../../platform.js";
import type { Profile } from "./types.js";
import { useModList, useProfile, useProfileCollection, useUpdateProfile } from "./query.js";

type Props = {
    onClose: () => void;
    profileId: string | null,
}

export const ModPanel = ({ onClose, profileId }: Props) => {
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
                                    <row>
                                        <text size={14}>Profile Settings</text>
                                    </row>
                                </button>
                                <button style={tab === "mods" ? "primary" : "secondary"} onPress={() => setTab("mods")}>
                                    <row>
                                        <text size={14}>Mods</text>
                                    </row>
                                </button>
                            </row>
                            <space height={18} />

                            {tab === "settings" ? <SettingsTab data={data!} /> : <ModsList profileId={profileId} />}
                        </col>
                    )
                }
            </view>
        </opaque>
    );
}

const ICONS: Icon[] = ["leaf", "sword", "flask", "crown", "rocket", "box"];
const SettingsTab = ({ data, }: { data: Profile }) => {
    const [name, setName] = useState(data.name);
    const profileCollection = useProfileCollection();
    const [profileDir, setProfileDir] = useState(data.dir);
    const [launchArgs, setLaunchArgs] = useState(data.launch_args);

    const update = useUpdateProfile(data.id);

    return (
        <>
            <col spacing={12}>
                <col spacing={4}>
                    <text size={13}>Name</text>
                    <input size={14}
                        padding={[8, 8]}
                        type="text"
                        placeholder="profile name"
                        value={name}
                        onChange={(ev) => {
                            setName(ev.value);
                        }}
                        onSubmit={() =>
                            update((draft) => {
                                draft.name = name;
                            })
                        } />
                </col>

                <col spacing={4}>
                    <text size={13}>Icon</text>
                    <row spacing={6}>
                        {ICONS.map((kind, i) => (
                            <IconButton
                                key={i}
                                type={kind}
                                active={data?.icon === kind}
                                onPress={() => update((draf) => { draf.icon = kind; })} />
                        ))}
                    </row>
                </col>

                <col spacing={4}>
                    <text size={13}>Launch Arguments</text>
                    <input size={14}
                        padding={[8, 8]}
                        type="text"
                        placeholder="-windowed -console"
                        value={launchArgs}
                        onChange={(ev) => setLaunchArgs(ev.value)}
                        onSubmit={() => update(draf => {
                            draf.launch_args = launchArgs;
                        })}
                    />
                </col>

                <col spacing={4}>
                    <text size={13}>Profile folder</text>
                    <row spacing={4}>
                        <input
                            size={14}
                            padding={[8, 8]}
                            type="text"
                            placeholder="path://"
                            value={profileDir}
                            onChange={(ev) => {
                                setProfileDir(ev.value);
                            }}
                            onSubmit={() => {
                                update(draf => {
                                    draf.dir = profileDir;
                                })
                            }}
                        />
                        <button width="shrink" onPress={() => {
                            void openDialog({ start: profileDir }).then((result) => {
                                if (!result.path) return;
                                setProfileDir(result.path);
                                update(draf => {
                                    draf.dir = result.path!;
                                });
                            }).catch((err) => {
                                console.error(err);
                            });
                        }}>
                            <svg style="currentColor" src={FolderInput} />
                        </button>
                    </row>
                </col>
            </col>

            <space height="fill" />
            <hr />
            <space height={6} />
            <row padding={4} alignX="center" alignY="center">
                <text size={13}>Deletes the profile folder</text>
                <space width="fill" />
                <button style="danger" width="shrink" onPress={() => {
                    confirmDialog({
                        title: "Delete Profile?",
                        description: "Are you sure you want to delete this profile?",
                        buttons: "OkCancel",
                        level: "warning"
                    }).then(ev => {
                        if (ev.result !== "Ok") return;
                        profileCollection.delete(data.id);
                    }).catch(err => console.error(err));
                }}>
                    <row alignX="center" alignY="center" spacing={4}>
                        <svg height={15} width={15} style="currentColor" src={Trash} />
                        <text size={14}>Delete profile</text>
                    </row>
                </button>
            </row>
        </>
    );
}

const ModsList = ({ profileId }: { profileId: string; }) => {
    const [query, setQuery] = useState("");
    const { data: mods } = useModList(profileId);
    const update = useUpdateProfile(profileId);

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
                                        update(draf => {
                                            const mod = draf.mods.findIndex(mod => mod.id === e.id);
                                            if (mod !== -1 && draf.mods[mod]) draf.mods[mod].enabled = ev.value;
                                        })
                                    }} />
                                    <button width={30} style="subtle" height={30} padding={6} onPress={() => {
                                        update(draf => {
                                            draf.mods = draf.mods.filter(mod => mod.id !== e.id);
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