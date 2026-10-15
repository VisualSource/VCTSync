import Play from "../../assets/icons/play.svg" with { type: "svg" };
import { AnimActivity, invoke } from "react-iced-native";
import { useApp } from "../../state.js";
import { ModPanel } from "./mod-panel.js";
import { useRef, useState } from "react";
import { ProfileList } from "./profile-list.js";
import { useProfileCollection, useProfileOptions } from "./query.js";

export const Profiles = () => {
    const [detailsId, setDetailsId] = useState<string | null>(null);
    const [showDetails, setShowDetails] = useState(false);
    const activeProfile = useApp(state => state.activeProfile);
    const setActiveProfile = useApp(state => state.setActiveProfile);

    const options = useProfileOptions();
    const collection = useProfileCollection();
    const pendingDelete = useRef<string | null>(null);

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
                <ProfileList toggleShowDetails={(id: string) => { setDetailsId(id); setShowDetails(true); }} />
            </col>


            <AnimActivity active={showDetails} from={{ x: "100%" }} to={{ x: 0 }} duration={220} onSettle={(ev) => {
                // Delete only once the panel has slid out, so it never renders a missing record.
                if (ev.value || !pendingDelete.current) return;
                collection.delete(pendingDelete.current);
                pendingDelete.current = null;
                setDetailsId(null);
            }}>
                <ModPanel
                    onClose={() => setShowDetails(false)}
                    onDelete={(id) => { pendingDelete.current = id; setShowDetails(false); }}
                    profileId={detailsId} />
            </AnimActivity>
        </stack>
    )
}