import { useState } from "react";
import Trash from "@icon/trash.svg" with {type: "svg" };
import XIcon from "@icon/x.svg" with {type: "svg" };
import FolderInput from "@icon/folder-input.svg" with {type: "svg"};
import { IconButton } from "./profile-icon.js";
import { useMutation } from "@tanstack/react-query";
import { confirmDialog, openDialog } from "../../platform.js";

type Props = {
    onClose: () => void;
}

type MutationArgs = {
    type: "icon",
    value: string;
} | { type: "name", value: string } |
{ type: "args", value: string; } |
{ type: "profile_dir", value: string; } | { type: "delete" }

export const ModPanel = ({ onClose }: Props) => {
    const [tab, setTab] = useState("settings");

    const { mutate } = useMutation({
        mutationFn: async (args: MutationArgs) => { },
        onError: () => { },
        mutationKey: ["profile"]
    });

    return (
        <opaque>
            <view style={{ background: "background" }} height="fill">
                <col padding={4}>
                    <row alignX="center" alignY="center">
                        <text>Profile Name</text>
                        <space width="fill" />
                        <button width="shrink" style="subtle" onPress={onClose}>
                            <svg height={18} width={18} src={XIcon} style="currentColor" />
                        </button>
                    </row>
                    <space height={4} />
                    <row spacing={4}>
                        <button style={tab === "settings" ? "primary" : "secondary"} onPress={() => setTab("settings")}>Profile Settings</button>
                        <button style={tab === "mods" ? "primary" : "secondary"} onPress={() => setTab("mods")}>Mods</button>
                    </row>
                    <hr />

                    <col>
                        <text size={13}>Icon</text>
                        <row spacing={6}>
                            <IconButton type="leaf" active onPress={() => mutate({ type: "icon", value: "leaf" })} />
                            <IconButton type="sword" onPress={() => mutate({ type: "icon", value: "sword" })} />
                            <IconButton type="flask" onPress={() => mutate({ type: "icon", value: "flask" })} />
                            <IconButton type="crown" onPress={() => mutate({ type: "icon", value: "crown" })} />
                            <IconButton type="rocket" onPress={() => mutate({ type: "icon", value: "rocket" })} />
                            <IconButton type="box" onPress={() => mutate({ type: "icon", value: "box" })} />
                        </row>
                    </col>

                    <col>
                        <text size={13}>Name</text>
                        <input type="text" placeholder="profile name" value={""} onChange={(ev) => {

                            mutate({ type: "name", value: ev.value })

                        }} />
                    </col>

                    <col>
                        <text size={13}>Launch Arguments</text>
                        <input type="text" placeholder="-windowed -console" value={""} onChange={(ev) => {
                            mutate({ type: "args", value: ev.value });
                        }} />
                    </col>

                    <col>
                        <text size={13}>Profile folder</text>
                        <row>
                            <input type="text" placeholder="path://" value={""} />
                            <button width="shrink" onPress={() => {
                                void openDialog({ start: "" }).then((result) => {
                                    if (!result.path) return;
                                    mutate({ type: "profile_dir", value: result.path });
                                }).catch((err) => {
                                    console.error(err);
                                });
                            }}>
                                <svg style="currentColor" src={FolderInput} />
                            </button>
                        </row>
                    </col>

                    <space height="fill" />

                    <hr />
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
                            <row alignX="center" alignY="center">
                                <svg height={15} width={15} style="currentColor" src={Trash} />
                                <text size={14}>Delete Profile</text>
                            </row>
                        </button>
                    </row>
                </col>
            </view>
        </opaque>
    );
}