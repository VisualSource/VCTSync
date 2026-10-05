import { useState } from "react";

export const Settings = () => {
    const [agentAddress, setAgentAddress] = useState("");
    const [token, setToken] = useState("");
    const [saveDir, setSaveDir] = useState("");

    return (
        <col>
            <text>Dev Agent</text>
            <col>
                <text>Agent Address</text>
                <input type="text" placeholder="192.168.1.94:9787" value={agentAddress} onChange={(ev) => setAgentAddress(ev.value)} />

                <text>Token</text>
                <input type="text" placeholder="shared bearer token" value={token} onChange={(ev) => setToken(ev.value)} />

                <text>Save dir (default downloads)</text>
                <input type="text" placeholder="C:\\Downloads" value={saveDir} onChange={(ev) => setSaveDir(ev.value)} />

                <space height={8} />
            </col>

            <button>
                Save Settings
            </button>
        </col>
    );
}