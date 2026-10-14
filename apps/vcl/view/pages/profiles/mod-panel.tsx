import { useState } from "react";
import Trash from "@icon/trash.svg" with {type: "svg" };
import XIcon from "@icon/x.svg" with {type: "svg" };
import FolderInput from "@icon/folder-input.svg" with {type: "svg"};
import { IconButton, ProfileIcon } from "./profile-icon.js";
import { useMutation, useQuery, useSuspenseQuery } from "@tanstack/react-query";
import { confirmDialog, openDialog } from "../../platform.js";
import type { Mod, Profile } from "./types.js";
type Props = {
    onClose: () => void;
    profileId: string | null,
}

type MutationArgs = {
    type: "icon" | "name" | "args" | "dir",
    value: string;
} |
{ type: "delete" }

export const ModPanel = ({ onClose, profileId }: Props) => {
    const [tab, setTab] = useState("settings");

    const { data, isError, isLoading, error } = useQuery({
        queryKey: ["profile", profileId],
        queryFn: async ({ queryKey, client }) => {
            const id = queryKey.at(1);

            if (!id) throw new Error("Unknown profile id");

            const profiles = client.getQueryData<Profile[]>(["profiles"]);

            const profile = profiles?.find(e => e.id === id);
            if (!profile) throw new Error("failed to find profile id");

            return profile;
        }
    });

    const { mutate } = useMutation<void, Error, MutationArgs, { previous: Profile[] | undefined }>({
        async onMutate(variables, context) {
            await context.client.cancelQueries({ queryKey: ["profiles"] });
            await context.client.cancelQueries({ queryKey: ["profile", profileId] });

            // ["profile", id] is derived from ["profiles"], so the list is the source of truth
            const previous = context.client.getQueryData<Profile[]>(["profiles"]);

            if (previous && variables.type !== "delete") {
                const key = variables.type === "args" ? "launch_args" : variables.type;
                const next = previous.map(p => p.id === profileId ? { ...p, [key]: variables.value } : p);
                context.client.setQueryData(["profiles"], next);
                context.client.setQueryData(["profile", profileId], next.find(p => p.id === profileId));
            }

            return { previous };
        },
        mutationFn: async (args, context) => {
            //TODO: save changes

        },
        onError(error, variables, onMutateResult, context) {
            console.error(error);
            context.client.setQueryData(["profiles"], onMutateResult?.previous);
            context.client.invalidateQueries({ queryKey: ["profile", profileId] });
        },
        onSettled(data, error, variables, onMutateResult, context) {
            // ponytail: ["profiles"] not invalidated, its mock queryFn would revert every edit; add it back once mutationFn saves
            context.client.invalidateQueries({ queryKey: ["profile", profileId] });
        },
        mutationKey: ["profile-update"]
    });

    return (
        <opaque>
            <view style={{ background: "background" }} height="fill">
                {
                    isLoading ? (
                        <col></col>
                    ) : isError ? (
                        <col alignX="center" alignY="center">
                            <text>{error?.message ?? String(error)}</text>
                        </col>
                    ) : (
                        <col padding={4}>
                            <row alignX="center" alignY="center">
                                <row spacing={6} alignX="center" alignY="center">
                                    <ProfileIcon type={data!?.icon} />
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

                            {tab === "settings" ? <SettingsTab mutate={mutate} data={data!} /> : <ModsList mutate={mutate} mods={data?.mods ?? []} />}
                        </col>
                    )
                }
            </view>
        </opaque>
    );
}

const SettingsTab = ({ data, mutate }: { data: Profile, mutate: (args: MutationArgs) => void }) => {
    const [name, setName] = useState(data.name);
    const [profileDir, setProfileDir] = useState(data.dir);
    const [launchArgs, setLaunchArgs] = useState(data.launch_args);

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
                        onSubmit={() => mutate({ type: "name", value: name })} />
                </col>

                <col spacing={4}>
                    <text size={13}>Icon</text>
                    <row spacing={6}>
                        <IconButton type="leaf" active={data?.icon === "leaf"} onPress={() => mutate({ type: "icon", value: "leaf" })} />
                        <IconButton type="sword" active={data?.icon === "sword"} onPress={() => mutate({ type: "icon", value: "sword" })} />
                        <IconButton type="flask" active={data?.icon === "flask"} onPress={() => {
                            mutate({ type: "icon", value: "flask" });
                        }} />
                        <IconButton type="crown" active={data?.icon === "crown"} onPress={() => {
                            mutate({ type: "icon", value: "crown" });
                        }} />
                        <IconButton type="rocket" active={data?.icon === "rocket"} onPress={() => {
                            mutate({ type: "icon", value: "rocket" });
                        }} />
                        <IconButton type="box" active={data?.icon === "box"} onPress={() => {
                            mutate({ type: "icon", value: "box" });
                        }} />
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
                        onSubmit={() => mutate({ type: "args", value: launchArgs })}
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
                                mutate({ type: "dir", value: profileDir });
                            }}
                        />
                        <button width="shrink" onPress={() => {
                            void openDialog({ start: profileDir }).then((result) => {
                                if (!result.path) return;
                                setProfileDir(result.path);
                                mutate({ type: "dir", value: result.path });
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
                        mutate({ type: "delete" })
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

const ModsList = ({ mods, mutate }: { mods: Mod[], mutate: (args: MutationArgs) => void; }) => {
    const [query, setQuery] = useState("");

    const filtered = mods;

    return (
        <>
            <row>
                <input type="text" value={query} onChange={(ev) => setQuery(ev.value)} placeholder="Search mods" />
            </row>

            <space height={10} />

            {filtered.length === 0 ? (
                <col height="fill" alignX="center" alignY="center">
                    <view>
                        <text>No mods</text>
                    </view>
                </col>
            ) : (
                <scroll>
                    <col spacing={6}>
                        {filtered.map(e => (
                            <view key={e.id} style={{ background: "background.weaker", borderRadius: 4 }}>
                                <row padding={6} spacing={6} alignY="center">
                                    <view height={30} width={30}>
                                        <svg src={Trash} />
                                    </view>
                                    <col>
                                        <text size={14}>{e.name}</text>
                                        <text size={12}>{e.author}</text>
                                    </col>

                                    <space width="fill" />

                                    <input type="switch" checked />
                                    <button width="shrink" style="subtle">
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