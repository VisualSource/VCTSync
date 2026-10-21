import { create } from "zustand";
import { invoke } from "react-iced-native";

/** Mirrors `AppSettings` in src/state.rs; stored in settings.ron. */
export type Settings = {
    /** iced theme name; "" is Dark. */
    theme: string;
    /** Steam folder override; "" auto-detects. */
    steamDir: string;
    launchMethod: "steam" | "direct" | "";
    modOnlyLogLines: boolean;
    devApiUrl: string;
    devApiToken: string;
    saveDir: string;
}

const DEFAULT_SETTINGS: Settings = {
    theme: "",
    steamDir: "",
    launchMethod: "",
    modOnlyLogLines: false,
    devApiUrl: "",
    devApiToken: "",
    saveDir: "",
};

export type Page = "profiles" | "builds" | "logs" | "settings";

type AppState = Settings & {
    activeProfile: string | null;
    /** The rail page being shown. */
    page: Page;
    setPage: (page: Page) => void;

    /** Settings came back from the host; edits before this aren't saved. */
    loaded: boolean;
    saveState: "saved" | "saving" | "error";
    saveError: string | null;

    loadSettings: () => Promise<void>;
    /** Applies at once; saved to the host 400ms after the last change. */
    updateSettings: (patch: Partial<Settings>) => void;
    setActiveProfile: (id: string | null) => void;
}

const pickSettings = (state: AppState): Settings =>
    Object.fromEntries(Object.keys(DEFAULT_SETTINGS).map(key => [key, state[key as keyof Settings]])) as Settings;

let saveTimer: ReturnType<typeof setTimeout> | undefined;

export const useApp = create<AppState>()((set, get) => ({
    ...DEFAULT_SETTINGS,
    activeProfile: null,
    page: "logs",
    setPage(page) {
        set({ page });
    },

    loaded: false,
    saveState: "saved",
    saveError: null,

    async loadSettings() {
        try {
            const settings = await invoke<Partial<Settings>>("fetch::settings", {});
            set({ ...DEFAULT_SETTINGS, ...settings, loaded: true });
        } catch (err) {
            console.error(err);
            set({ loaded: true });
        }
    },
    updateSettings(patch) {
        set(patch);
        if (!get().loaded) return;

        set({ saveState: "saving" });
        clearTimeout(saveTimer);
        saveTimer = setTimeout(() => {
            invoke<{ error?: string }>("save::settings", pickSettings(get()))
                .then(result => set(result.error ? { saveState: "error", saveError: result.error } : { saveState: "saved", saveError: null }))
                .catch((err: Error) => set({ saveState: "error", saveError: err.message }));
        }, 400);
    },
    setActiveProfile(id) {
        set({ activeProfile: id })
    },
}));
