export const ModDetailPanel = () => {

    return (
        <opaque>
            <view>
                <col>
                    <row>
                        <button></button>
                        <text></text>
                        <space width="fill" />
                        <button></button>
                    </row>

                    <row>
                        <view></view>

                        <col>
                            <text>Name</text>
                            <text>other * version</text>
                            <row>
                                <text>DOwnloads</text>
                                <text>Rating</text>
                                <text>Updated</text>
                            </row>
                        </col>
                    </row>

                    <text>Description</text>

                    <text>DEPENDENCIES (0)</text>
                    <scroll height={200}>
                        <col></col>
                    </scroll>

                    <space height="fill" />

                    <hr />
                    <text>Status</text>

                    <button>Install</button>

                </col>
            </view>
        </opaque>
    );
}