import type { Icon } from "./profile-icon.js";

export type Mod = {
    name: string;
    id: string;
    icon: string;
    enabled: boolean;
    author: string;
}


export type Profile = {
    dir: string;
    launch_args: string;
    name: string;
    id: string;
    icon: Icon,
    mods: Mod[]
}