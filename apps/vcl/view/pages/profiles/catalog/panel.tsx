import BackArrow from "@icon/cast.svg";
import Archive from "@icon/box.svg";
import Refresh from "@icon/rocket.svg";
import { useState } from "react";

const SELECT_OPTIONS = [{
    label: "Popular",
    value: "popular"
}]

export const Catalog = () => {
    const [filterBy, setFilterBy] = useState("popular");
    const [query, setQuery] = useState("");

    return (
        <opaque>
            <view style={{ background: "background" }} height="fill">
                <col>
                    <row>
                        <button>
                            <svg src={BackArrow} />
                        </button>
                        <col>
                            <text>Add Mods</text>
                            <text>Back</text>
                        </col>

                        <space width="fill" />

                        <button>
                            <row>
                                <svg src={Archive} />
                                <text>From .zip</text>
                            </row>
                        </button>
                    </row>

                    <row>
                        <input type="text" onChange={(ev) => setQuery(ev.value)} value={query} />
                        <select onSelect={(ev) => setFilterBy(ev.value)} options={SELECT_OPTIONS} value={filterBy} />
                    </row>

                    <scroll height="fill" spacing={2}>
                        <col>

                        </col>
                    </scroll>

                    <hr />
                    <row>
                        <text>synced 3 min ago</text>
                        <space width="fill" />
                        <button>
                            <svg src={Refresh} />
                        </button>
                    </row>
                </col>
            </view>
        </opaque>
    );
}