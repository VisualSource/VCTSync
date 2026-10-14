import type { Icon } from "./profile-icon.js";

export type Profile = {
    name: string;
    id: string;
    icon: Icon,
    mods: {
        name: string;
        icon: string;
        id: string;
    }[]
}