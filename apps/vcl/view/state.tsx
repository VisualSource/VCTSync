import { create } from "zustand";

type AppState = {
    devApiUrl: string;
    devApiToken: string;

    saveDir: string;

    modOnlyLogLines: boolean;

    setSaveDir: (value: string) => void;
    setDevApiToken: (value: string) => void;
    setDevApiUrl: (value: string) => void;
    setModOnlyLogLines: (value: boolean) => void;
}

export const useApp = create<AppState>()((set) => ({
    devApiUrl: "",
    devApiToken: "",

    saveDir: "",

    modOnlyLogLines: false,

    setSaveDir(value: string) {
        set({ saveDir: value })
    },
    setDevApiUrl(value: string) {
        set({ devApiUrl: value });
    },
    setDevApiToken(value: string) {
        set({ devApiToken: value });
    },
    setModOnlyLogLines(value: boolean) {
        set({ modOnlyLogLines: value });
    }
}));