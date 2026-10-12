import { useApp } from "../state.js";

export const Settings = () => {
    const { devApiToken, devApiUrl, saveDir, setDevApiToken, setDevApiUrl, setSaveDir } = useApp();

    return (
        <col>
            <text>Dev Agent</text>
            <col>
                <text>Agent Address</text>
                <input type="text" placeholder="192.168.1.94:9787" value={devApiUrl} onChange={(ev) => setDevApiUrl(ev.value)} />

                <text>Token</text>
                <input type="text" placeholder="shared bearer token" value={devApiToken} onChange={(ev) => setDevApiToken(ev.value)} />

                <text>Save dir (default downloads)</text>
                <input type="text" placeholder="C:\\Downloads" value={saveDir} onChange={(ev) => setSaveDir(ev.value)} />

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