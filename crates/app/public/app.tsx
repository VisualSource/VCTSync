import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { useState } from "react";

import { Tooltip } from "./utils.js";

import { Builds } from "./pages/builds.js";
import { Logs } from "./pages/logs.js";
import { Profiles } from "./pages/profiles.js";
import { Settings } from "./pages/settings.js";

import Library from "./assets/icons/library.svg";
import Boxes from "./assets/icons/boxes.svg";
import ScrollText from "./assets/icons/scroll-text.svg";
import SettingsSvg from "./assets/icons/settings.svg";

const client = new QueryClient();



const App = () => {
    const [tab, setTab] = useState<string>("logs");

    return (
        <QueryClientProvider client={client}>
            <row>
                <col spacing={4}>
                    <Tooltip tip="Profiles">
                        <button onPress={() => setTab("profiles")}>
                            <svg width={18} height={18} src={Library} />
                        </button>
                    </Tooltip>
                    <Tooltip tip="Builds">
                        <button onPress={() => setTab("builds")}>
                            <svg width={18} height={18} src={Boxes} />
                        </button>
                    </Tooltip>
                    <Tooltip tip="Logs">
                        <button onPress={() => setTab("logs")}>
                            <svg width={18} height={18} src={ScrollText} />
                        </button>
                    </Tooltip>
                    <space height="fill" />

                    <Tooltip tip="Settings">
                        <button onPress={() => setTab("settings")}>
                            <svg width={18} height={18} src={SettingsSvg} />
                        </button>
                    </Tooltip>

                    <space height={4} />
                </col>
                <vr />
                <space width={4} />
                <view>
                    <View tab={tab} />
                </view>
            </row>
        </QueryClientProvider>
    );
}

const View = ({ tab }: { tab: string }) => {
    switch (tab) {
        case "builds":
            return <Builds />;
        case "logs":
            return <Logs />;
        case "profiles":
            return <Profiles />
        case "settings":
            return <Settings />
        default:
            return null;
    }
}

