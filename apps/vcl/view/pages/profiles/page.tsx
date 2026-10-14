import { useQuery } from "@tanstack/react-query";

import Play from "../../assets/icons/play.svg" with { type: "svg" };
import { AnimActivity, invoke } from "react-iced-native";
import { useApp } from "../../state.js";
import { ModPanel } from "./mod-panel.js";
import { useState } from "react";
import type { Profile } from "./types.js";
import { ProfileList } from "./profile-list.js";

export const Profiles = () => {
    const [detailsId, setDetailsId] = useState<string | null>(null);
    const [showDetails, setShowDetails] = useState(false);
    const activeProfile = useApp(state => state.activeProfile);
    const setActiveProfile = useApp(state => state.setActiveProfile);
    const { isLoading, isError, error, data } = useQuery({
        queryKey: ["profiles"],
        queryFn: async () => {
            return [{ name: "Example", id: "test example", icon: "leaf", mods: [{ name: "BepinEx", author: "BepinEx", enabled: true, icon: "somepath", id: "adeddd" }], dir: "C:\\Repos", launch_args: "" }] as Profile[];
        }
    });

    const options = data?.map(e => ({ label: e.name, value: e.id })) ?? [];

    return (
        <stack>
            <col height="fill">
                <row padding={6}>
                    <row spacing={2} alignY="center">
                        <select options={options} placeholder="Select profile" onSelect={(ev) => setActiveProfile(ev.value)} value={activeProfile ?? undefined} />
                    </row>
                    <space width="fill" />
                    <button onPress={() => {
                        void invoke("launch-game", {}).catch((err) => {
                            console.error(err);
                        })
                    }}>
                        <row alignX="center" alignY="center">
                            <svg height={15} width={15} style="currentColor" src={Play} />
                            <text>Launch</text>
                        </row>
                    </button>
                </row>
                <hr />
                <space height={10} />
                <ProfileList toggleShowDetails={(id: string) => { setDetailsId(id); setShowDetails(true); }} activeProfile={activeProfile} isError={isError} isLoading={isLoading} error={error} data={data} />
            </col>


            <AnimActivity active={showDetails} from={{ x: "100%" }} to={{ x: 0 }} duration={220}>
                <ModPanel onClose={() => setShowDetails(false)} profileId={detailsId} />
            </AnimActivity>
        </stack>
    )
}