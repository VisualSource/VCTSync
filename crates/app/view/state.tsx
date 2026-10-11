import { create } from "zustand";

type AppState = {
    devApiUrl: string;
    devApiToken: string;

    saveDir: string;

    setSaveDir: (value: string) => void;
    setDevApiToken: (value: string) => void;
    setDevApiUrl: (value: string) => void;
}

export const useApp = create<AppState>()((set) => ({
    devApiUrl: "",
    devApiToken: "",

    saveDir: "",

    setSaveDir(value: string) {
        set({ saveDir: value })
    },
    setDevApiUrl(value: string) {
        set({ devApiUrl: value });
    },
    setDevApiToken(value: string) {
        set({ devApiToken: value });
    }
}));