import { QueryClientProvider } from "@tanstack/react-query";
import { createRoot } from "react-iced-native";
import { useState } from "react";

import { Tooltip } from "./utils.js";

import { Builds } from "./pages/builds/page.js";
import { Logs } from "./pages/logs/page.js";
import { Profiles } from "./pages/profiles/page.js";
import { Settings } from "./pages/settings.js";

import Library from "./assets/icons/library.svg" with {type: "svg" };
import Boxes from "./assets/icons/boxes.svg" with {type: "svg" };
import ScrollText from "./assets/icons/scroll-text.svg" with {type: "svg" };
import SettingsSvg from "./assets/icons/settings.svg" with {type: "svg" };
import { queryClient, dbClient } from "./client.js";
import { DbProvider } from "@tanstack/react-db";
import { useApp } from "./state.js";

const App = () => {
    const [tab, setTab] = useState<string>("logs");

    return (
        <QueryClientProvider client={queryClient}>
            <DbProvider client={dbClient}>
                <row>
                    <col spacing={4}>
                        <Tooltip tip="Profiles">
                            <button onPress={() => setTab("profiles")}>
                                <svg color="background.text" width={18} height={18} src={Library} />
                            </button>
                        </Tooltip>
                        <Tooltip tip="Builds">
                            <button onPress={() => setTab("builds")}>
                                <svg color="background.text" width={18} height={18} src={Boxes} />
                            </button>
                        </Tooltip>
                        <Tooltip tip="Logs">
                            <button onPress={() => setTab("logs")}>
                                <svg color="background.text" width={18} height={18} src={ScrollText} />
                            </button>
                        </Tooltip>
                        <space height="fill" />

                        <Tooltip tip="Settings">
                            <button onPress={() => setTab("settings")}>
                                <svg color="background.text" width={18} height={18} src={SettingsSvg} />
                            </button>
                        </Tooltip>

                        <space height={4} />
                    </col>
                    <vr />
                    <space width={4} />
                    <view>
                        <Router tab={tab} />
                    </view>
                </row>
            </DbProvider>
        </QueryClientProvider>
    );
}

const Router = ({ tab }: { tab: string }) => {
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



void useApp.getState().loadSettings();
createRoot("main").render(<App />);