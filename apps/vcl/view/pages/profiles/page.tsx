import Play from "../../assets/icons/play.svg" with { type: "svg" };
import { AnimActivity, invoke, listen } from "react-iced-native";
import { useEffect, type ReactNode } from "react";
import { useMutation } from "@tanstack/react-query";
import { message } from "../../platform/dialog.js";
import { useApp } from "../../state.js";
import { ModPanel } from "./details/panel.js";
import { ProfileList } from "./profile-list.js";
import { useProfile, useProfileCollection, useProfileOptions } from "./query.js";
import { useProfileState } from "./state.js";
import { Catalog } from "./catalog/panel.js";
import { ZipInstallPanel } from "./zip-install-panel.js";
import { ModDetailPanel } from "./catalog/mod-details.js";

/** Right-aligned pushed screen, as wide as the mockup's panel; clicks beside it reach the scrim. */
const Sheet = ({ children }: { children: ReactNode }) => (
    <view width="fill" height="fill" alignX="right">
        <view maxWidth={440} width="fill" height="fill">{children}</view>
    </view>
);

export const Profiles = () => {
    const { showCatalog, showDetails, clearDetails, showCatalogSubmenu, detailId, back, toggleDetails } = useProfileState();
    useEffect(() => listen("key::escape", back), [back]);
    const activeProfile = useApp(state => state.activeProfile);
    const setActiveProfile = useApp(state => state.setActiveProfile);

    const options = useProfileOptions();
    const collection = useProfileCollection();
    const { data: active } = useProfile(activeProfile);

    const launch = useMutation({
        mutationFn: async () => {
            if (!active) throw new Error("No active profile.");
            const result = await invoke<{ error?: string }>("launch-game", { profile_dir: active.dir, launch_args: active.launch_args });
            if (result.error) throw new Error(result.error);
        },
        onError: (err) => void message(err.message, { title: "Launch failed", buttons: "Ok", kind: "error" }).catch(console.error),
    });

    return (
        <stack>
            <col height="fill">
                <row padding={6}>
                    <row spacing={2} alignY="center">
                        {options.length ? (
                            <select options={options} placeholder="Select profile" onSelect={(ev) => setActiveProfile(ev.value)} value={activeProfile ?? undefined} />
                        ) : null}
                    </row>
                    <space width="fill" />
                    <button onPress={() => launch.mutate()} disabled={!active || launch.isPending}>
                        <row alignY="center" spacing={4}>
                            <svg height={15} width={15} color="background.text" src={Play} opacity={!active || launch.isPending ? 0.5 : 1} />
                            <text>{launch.isPending ? `Launching ${active?.name}…` : "Launch"}</text>
                        </row>
                    </button>
                </row>
                <hr />
                <space height={10} />
                <ProfileList />
            </col>

            {/* Always occupies this stack slot: iced keys widget state by position, so a vanishing child
                would shift the drawers below and reset their slide-out animation. */}
            {showDetails ? (
                <button width="fill" height="fill" padding={0} style={{ background: "#00000059", hoverBackground: "#00000059", pressedBackground: "#00000059" }} onPress={() => toggleDetails({ state: false })}>
                    <space />
                </button>
            ) : <space />}

            <AnimActivity active={showDetails} from={{ x: "100%" }} to={{ x: 0 }} duration={220} onSettle={(ev) => {
                const state = useProfileState.getState();
                // Delete only once the panel has slid out, so it never renders a missing record.
                if (ev.value || !state.pendingDelete) return;
                const id = state.pendingDelete;
                clearDetails();
                const profile = collection.get(id);
                if (!profile) return;

                // Folder first: if it can't be removed (e.g. game running) the profile stays so it can be retried.
                invoke<{ deleted?: boolean, dir?: string, error?: string }>("profiles::delete-dir", { profile_dir: profile.dir })
                    .then(async result => {
                        if (result.error) throw new Error(result.error);
                        collection.delete(id);
                        if (id === useApp.getState().activeProfile) setActiveProfile(collection.toArray.find(p => p.id !== id)?.id ?? null);
                        if (!result.deleted) await message(`The profile folder is outside the app folder, so it was left in place:\n${result.dir}`, { title: "Profile deleted", buttons: "Ok", kind: "info" });
                    })
                    .catch((err: Error) => message(`The profile folder couldn't be deleted, so the profile was kept.\n${err.message}`, { title: "Delete failed", buttons: "Ok", kind: "error" }))
                    .catch(console.error);
            }}>
                <Sheet><ModPanel key={detailId} /></Sheet>
            </AnimActivity>

            <AnimActivity active={showCatalog} from={{ x: "100%" }} to={{ x: 0 }} duration={220}>
                <Sheet><Catalog /></Sheet>
            </AnimActivity>


            {showCatalog ? (
                <>
                    <AnimActivity active={showCatalogSubmenu === "mod-detail"} from={{ x: "100%" }} to={{ x: 0 }} duration={220}>
                        <Sheet><ModDetailPanel /></Sheet>
                    </AnimActivity>

                    <AnimActivity active={showCatalogSubmenu === "zip-install"} from={{ x: "100%" }} to={{ x: 0 }} duration={220}>
                        <Sheet><ZipInstallPanel /></Sheet>
                    </AnimActivity>
                </>
            ) : null}
        </stack>
    )
}