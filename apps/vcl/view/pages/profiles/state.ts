import { create } from "zustand";

type Store = {
    showDetails: boolean;
    detailId: string | null;
    showCatalog: boolean;

    pendingDelete: string | null;

    toggleDetails: (opt?: { state?: boolean, pendingId?: string }) => void;
    toggleCatlog: (state?: boolean) => void;

    clearDetails: () => void;
    openDetails: (id: string) => void;
}

export const useProfileState = create<Store>()((set, get) => ({
    showDetails: false,
    showCatalog: false,

    detailId: null,
    pendingDelete: null,

    toggleDetails(opts) {
        const value = opts?.state !== undefined ? opts.state : !get().showDetails;

        const next: Partial<Store> = { showDetails: value };
        if (opts?.pendingId !== undefined) next.pendingDelete = opts.pendingId;

        set(next);

    },
    toggleCatlog(state) {
        const value = state !== undefined ? state : !get().showCatalog;
        set({ showCatalog: value });
    },
    clearDetails() {
        set({ pendingDelete: null, detailId: null });
    },
    openDetails(id) {
        set({ detailId: id, showDetails: true, showCatalog: false })
    },
}));