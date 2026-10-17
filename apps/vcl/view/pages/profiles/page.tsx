import Play from "../../assets/icons/play.svg" with { type: "svg" };
import { AnimActivity, invoke } from "react-iced-native";
import { useApp } from "../../state.js";
import { ModPanel } from "./details/panel.js";
import { ProfileList } from "./profile-list.js";
import { useProfileCollection, useProfileOptions } from "./query.js";
import { useProfileState } from "./state.js";
import { Catalog } from "./catalog/panel.js";
import { ZipInstallPanel } from "./zip-install-panel.js";
import { ModDetailPanel } from "./catalog/mod-details.js";

export const Profiles = () => {
    const { showCatalog, showDetails, clearDetails, showCatalogSubmenu } = useProfileState();
    const activeProfile = useApp(state => state.activeProfile);
    const setActiveProfile = useApp(state => state.setActiveProfile);

    const options = useProfileOptions();
    const collection = useProfileCollection();

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
                        <row alignX="center" alignY="center" spacing={2}>
                            <svg height={15} width={15} color="background.text" src={Play} />
                            <text>Launch</text>
                        </row>
                    </button>
                </row>
                <hr />
                <space height={10} />
                <ProfileList />
            </col>

            <AnimActivity active={showDetails} from={{ x: "100%" }} to={{ x: 0 }} duration={220} onSettle={(ev) => {
                const state = useProfileState.getState();
                // Delete only once the panel has slid out, so it never renders a missing record.
                if (ev.value || !state.pendingDelete) return;
                collection.delete(state.pendingDelete);
                clearDetails();
            }}>
                <ModPanel />
            </AnimActivity>

            <AnimActivity active={showCatalog} from={{ x: "100%" }} to={{ x: 0 }} duration={220}>
                <Catalog />
            </AnimActivity>


            {showCatalog ? (
                <>
                    <AnimActivity active={showCatalogSubmenu === "mod-detail"} from={{ x: "100%" }} to={{ x: 0 }} duration={220}>
                        <ModDetailPanel />
                    </AnimActivity>

                    <AnimActivity active={showCatalogSubmenu === "zip-install"} from={{ x: "100%" }} to={{ x: 0 }} duration={220}>
                        <ZipInstallPanel />
                    </AnimActivity>
                </>
            ) : null}
        </stack>
    )
}