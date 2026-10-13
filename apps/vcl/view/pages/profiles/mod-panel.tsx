import { useState } from "react";
import Trash from "../../assets/icons/trash.svg" with {type: "svg" };
import XIcon from "../../assets/icons/x.svg" with {type: "svg" };
type Props = {
    onClose: () => void;
}

export const ModPanel = ({ onClose }: Props) => {
    const [tab, setTab] = useState("settings");
    return (
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
                    <button style={tab === "settings" ? "primary" : "secondary"}>Profile Settings</button>
                    <button style={tab === "mods" ? "primary" : "secondary"}>Mods</button>
                </row>

                <col>
                    <text size={13}>Name</text>
                    <input type="text" placeholder="profile name" value={""} />
                </col>

                <col>
                    <text size={13}>Icon</text>
                    <row></row>
                </col>

                <col>
                    <text size={13}>Launch Arguments</text>
                    <input type="text" placeholder="-windowed -console" value={""} />
                </col>

                <col>
                    <text size={13}>Profile folder</text>
                    <input type="text" placeholder="path://" value={""} />
                </col>

                <space height="fill" />

                <hr />
                <row padding={4} alignX="center" alignY="center">
                    <text size={13}>Deletes the profile folder</text>
                    <space width="fill" />
                    <button style="danger" width="shrink">
                        <row alignX="center" alignY="center">
                            <svg style="currentColor" src={Trash} />
                            <text>Delete Profile</text>
                        </row>
                    </button>
                </row>
            </col>
        </view>
    );
}