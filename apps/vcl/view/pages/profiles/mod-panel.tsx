import Trash from "../../assets/icons/library.svg" with {type: "svg" };

type Props = {
    onClose: () => void;
}

export const ModPanel = ({ onClose }: Props) => {
    return (
        <view style={{ background: "" }}>
            <col>
                <row>
                    <text>Profile Name</text>
                    <space width="fill" />
                    <button onPress={onClose}>
                        X
                    </button>
                </row>
                <space height={4} />
                <row>

                </row>
                <hr />

                <col>
                    <text>Name</text>
                    <input type="text" placeholder="profile name" value={""} />
                </col>

                <col>
                    <text>Icon</text>
                    <row></row>
                </col>

                <col>
                    <text>Launch Arguments</text>
                    <input type="text" placeholder="-windowed -console" value={""} />
                </col>

                <col>
                    <text>Profile folder</text>
                    <input type="text" placeholder="path://" value={""} />
                </col>

                <space height="fill" />

                <hr />
                <row>
                    <text>Deletes the profile folder</text>
                    <button>
                        <row>
                            <svg src={Trash} />
                            <text>Delete Profile</text>
                        </row>
                    </button>
                </row>
            </col>
        </view>
    );
}