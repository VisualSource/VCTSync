import { collectionOptions, eq, useDbClient, useLiveQuery } from "@tanstack/react-db";
import { queryCollectionOptions } from '@tanstack/query-db-collection'
import type { Profile } from "./types.js";
import type { QueryClient } from "@tanstack/react-query";
import { invoke } from "react-iced-native";

const persist = async (profiles: Profile[]) => {
    const result = await invoke<{ error?: string }>("profiles::save", profiles);
    if (result.error) throw new Error(`Saving profiles failed: ${result.error}`);
}

// Each handler saves the full list (collection.toArray already includes the optimistic change)
// before confirming it; a failed save throws, which rolls the change back.
const profilesCollection = collectionOptions("profiles",
    (client) => queryCollectionOptions({
        id: "profiles",
        queryKey: ["profiles"],
        queryClient: client.requireDependency<QueryClient>("queryClient"),
        queryFn: async () => {
            const result = await invoke<Profile[] | { error: string }>("profiles::list", {});
            if ("error" in result) throw new Error(`Loading profiles failed: ${result.error}`);
            return result;
        },
        getKey: item => item.id,
        onInsert: async ({ transaction, collection }) => {
            await persist(collection.toArray);
            collection.utils.writeBatch(() => {
                for (const m of transaction.mutations) collection.utils.writeInsert(m.modified);
            });

            return { refetch: false }
        },
        onUpdate: async ({ transaction, collection }) => {
            const now = Temporal.Now.plainDateTimeISO().toString();
            const touched = new Set(transaction.mutations.map(m => m.key));
            await persist(collection.toArray.map(p => touched.has(p.id) ? { ...p, updated_at: now } : p));

            collection.utils.writeBatch(() => {
                for (const m of transaction.mutations) collection.utils.writeUpdate({ ...m.modified, updated_at: now });
            });

            return { refetch: false }
        },
        onDelete: async ({ transaction, collection }) => {
            await persist(collection.toArray);
            collection.utils.writeBatch(() => {
                for (const m of transaction.mutations) collection.utils.writeDelete(m.key);
            });

            return { refetch: false }
        }
    })
);

export const useProfileCollection = () => {
    return useDbClient().collection(profilesCollection);
}

export const useListProfiles = () => {
    const profilesCollection = useProfileCollection();

    const { data, persistedError, isError, isLoading } = useLiveQuery({
        query: p => p.from({ profile: profilesCollection })
    });

    return { data, error: persistedError, isError, isLoading };
}

export const useProfileOptions = () => {
    const profilesCollection = useProfileCollection();

    const { data } = useLiveQuery({
        query: p => p.from({ profile: profilesCollection }).select(({ profile }) => ({
            value: profile.id,
            label: profile.name
        }))
    })

    return data;
}

export const useProfile = (profileId: string | null) => {
    const profilesCollection = useProfileCollection();

    const { data, isLoading, isError, persistedError } = useLiveQuery({
        query: p => p.from({ profile: profilesCollection }).where(({ profile }) => eq(profile.id, profileId)).findOne()
    });

    return { data, isLoading, isError, error: persistedError }
}

export const useModList = (profileId: string | null, query?: string) => {
    const profilesCollection = useProfileCollection();

    const { data, isLoading, isError, persistedError } = useLiveQuery({
        query: p => p.from({
            profile: profilesCollection
        }).where(({ profile }) => eq(profile.id, profileId)).findOne()
    });

    const q = query?.trim().toLowerCase();
    const mods = q ? data?.mods.filter(e => e.name.toLowerCase().includes(q) || e.author.toLowerCase().includes(q)) : data?.mods;

    return { data: mods, isLoading, isError, persistedError };
}