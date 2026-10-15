import X from "@icon/x.svg" with { type: "svg" };

export const AddModDialog = ({ toggleAddMod, profileId }: { profileId: string | null, toggleAddMod: () => void }) => {

    return (
        <opaque>
            <view center="fill" height="fill" width="fill" style={{ background: "background.weakest", opacity: 0.5 }}>
                <view maxHeight={200} maxWidth={300} style={{ background: "background", shadowBlurRadius: 2, shadowColor: "#000000" }} padding={6}>
                    <col width="fill" height="fill">
                        <row>
                            <text>Add Mod</text>
                            <space width="fill" />
                            <button width="shrink" style="subtle" onPress={toggleAddMod}>
                                <svg height={18} width={18} src={X} color="background.text" />
                            </button>
                        </row>
                        <hr />




                        <row>
                            <button>
                                Ok
                            </button>
                        </row>
                    </col>
                </view>

            </view>
        </opaque>
    )


}