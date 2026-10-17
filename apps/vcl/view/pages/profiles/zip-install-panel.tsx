import { useEffect } from "react"

export const ZipInstallPanel = () => {
    return (
        <opaque>
            <view>
                <col>
                    <row>
                        <button>

                        </button>
                        <col>
                            <text>Add from file</text>
                            <text>Zip Path</text>
                        </col>
                    </row>

                    <row>
                        <view></view>
                        <col>
                            <text>Name</text>
                            <text>Version</text>
                        </col>
                    </row>

                    <row>
                        <text>Author</text>
                    </row>
                    <row>
                        <text>Website</text>
                    </row>

                    <text>Description</text>

                    <text>DEPENDENCIES (2)</text>
                    <scroll height={200}>
                        <col>
                            <row></row>
                        </col>
                    </scroll>
                    <space height="fill" />

                    <hr />
                    <text>All</text>
                    <row spacing={4}>
                        <button>Chosse another...</button>
                        <button>Add to profile</button>
                    </row>
                </col>
            </view>
        </opaque>
    );
}