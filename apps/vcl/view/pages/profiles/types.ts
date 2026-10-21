import type { Icon } from "./profile-icon.js";

export type Mod = {
    name: string;
    id: string;
    /** Local path to the installed icon.png, or "" if the package has none. */
    icon: string;
    enabled: boolean;
    author: string;
    version: string;
    description?: string;
    /** Thunderstore "Author-Name-Version" strings. */
    dependencies: string[];
    source: "local" | "thunderstore";
}


export type Profile = {
    dir: string;
    launch_args: string;
    name: string;
    id: string;
    icon: Icon,
    mods: Mod[],
    created_at: string;
    updated_at: string;
}