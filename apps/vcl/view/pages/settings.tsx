import { invoke } from "react-iced-native";
import { useApp } from "../state.js";
import { Tooltip } from "../utils.js";

import FolderInput from "../assets/icons/folder-input.svg" with { type: "svg" };

export const Settings = () => {
    const { devApiToken, devApiUrl, saveDir, setDevApiToken, setDevApiUrl, setSaveDir } = useApp();

    return (
        <col>
            <text font={{ weight: "bold" }}>Common</text>
            <hr />
            <space height={2} />
            <col>
                <text>Steam</text>
                <input type="text" placeholder="steam dir" value={""} />
            </col>

            <space height={8} />

            <text font={{ weight: "bold" }}>Dev Agent</text>
            <hr />
            <space height={2} />
            <col>
                <text>Agent Address</text>
                <input type="text" placeholder="192.168.1.94:9787" value={devApiUrl} onChange={(ev) => setDevApiUrl(ev.value)} />

                <text>Token</text>
                <input type="text" placeholder="shared bearer token" value={devApiToken} onChange={(ev) => setDevApiToken(ev.value)} />

                <text>Save dir (default downloads)</text>
                <row spacing={2} alignX="center">
                    <input type="text" placeholder="C:\\Downloads" value={saveDir} onChange={(ev) => setSaveDir(ev.value)} />
                    <Tooltip tip="Select folder" position="left">
                        <button width="shrink" onPress={() => {
                            void invoke<{ path: string | null; }>("open-folder", {}).then((result) => {
                                if (!result.path) return;
                                setSaveDir(result.path);
                            }).catch((err) => {
                                console.error(err);
                            });
                        }}>
                            <svg src={FolderInput} />
                        </button>
                    </Tooltip>

                </row>

                <space height={8} />
            </col>

            <button onPress={() => {
                /* TODO: save settings */
            }}>
                Save Settings
            </button>
        </col>
    );
}