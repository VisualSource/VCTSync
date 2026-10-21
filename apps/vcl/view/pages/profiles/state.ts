import { create } from "zustand";

type CatalogSubmenu = "zip-install" | "mod-detail"

type Store = {
    showDetails: boolean;
    detailId: string | null;
    showCatalog: boolean;
    showCatalogSubmenu: CatalogSubmenu | null;

    pendingModId: string | null;
    zipPath: string | null;

    pendingDelete: string | null;

    toggleDetails: (opt?: { state?: boolean, pendingId?: string }) => void;
    toggleCatlog: (state?: boolean) => void;

    clearDetails: () => void;
    openDetails: (id: string) => void;

    openModDetails: (modId: string) => void;
    openZipInstall: (path: string) => void;
    closeCatalogSubmenu: () => void;
    /** Pop the top screen: catalog submenu -> catalog -> profile panel. */
    back: () => void;
}

export const useProfileState = create<Store>()((set, get) => ({
    showDetails: false,
    showCatalog: false,
    showCatalogSubmenu: null,

    pendingModId: null,
    zipPath: null,
    detailId: null,
    pendingDelete: null,

    openModDetails(id) {
        set({ showCatalogSubmenu: "mod-detail", pendingModId: id });
    },
    openZipInstall(path) {
        set({ showCatalogSubmenu: "zip-install", zipPath: path });
    },
    // Ids stay set so the panel keeps rendering while it slides out.
    closeCatalogSubmenu() {
        set({ showCatalogSubmenu: null });
    },
    back() {
        const { showCatalogSubmenu, showCatalog, showDetails } = get();
        if (showCatalogSubmenu) set({ showCatalogSubmenu: null });
        else if (showCatalog) set({ showCatalog: false });
        else if (showDetails) set({ showDetails: false });
    },

    toggleDetails(opts) {
        const value = opts?.state !== undefined ? opts.state : !get().showDetails;

        const next: Partial<Store> = { showDetails: value };
        if (opts?.pendingId !== undefined) next.pendingDelete = opts.pendingId;

        set(next);

    },
    toggleCatlog(state) {
        const value = state !== undefined ? state : !get().showCatalog;
        set({ showCatalog: value, showCatalogSubmenu: null });
    },
    clearDetails() {
        set({ pendingDelete: null, detailId: null });
    },
    openDetails(id) {
        set({ detailId: id, showDetails: true, showCatalog: false })
    },
}));