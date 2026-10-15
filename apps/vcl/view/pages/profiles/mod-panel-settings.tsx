import { useState } from "react";
import FolderInput from "@icon/folder-input.svg" with {type: "svg"};
import Trash from "@icon/trash.svg" with {type: "svg" };
import { open, confirm } from "../../platform/dialog.js";
import { IconButton, ICONS } from "./profile-icon.js";
import { useProfileCollection } from "./query.js";
import type { Profile } from "./types.js";

export const SettingsTab = ({ data, onDelete }: { data: Profile, onDelete: (id: string) => void }) => {
    const [name, setName] = useState(data.name);
    const collection = useProfileCollection();
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
                        onSubmit={() => collection.update(data.id, draft => {
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
                                onPress={() => collection.update(data.id, (draft) => { draft.icon = kind; })} />
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
                        onSubmit={() => collection.update(data.id, draft => {
                            draft.launch_args = launchArgs;
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
                                collection.update(data.id, draft => {
                                    draft.dir = profileDir;
                                })
                            }}
                        />
                        <button width="shrink" onPress={() => {
                            void open({ defaultPath: profileDir }).then((result) => {
                                if (!result.path) return;
                                setProfileDir(result.path);
                                collection.update(data.id, draft => {
                                    draft.dir = result.path!;
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
                    confirm("Are you sure you want to delete this profile?", {
                        title: "Delete Profile?",
                        kind: "warning"
                    }).then(result => {
                        if (!result) return;
                        onDelete(data.id);
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