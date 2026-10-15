import { collectionOptions, eq, useDbClient, useLiveQuery } from "@tanstack/react-db";
import { queryCollectionOptions } from '@tanstack/query-db-collection'
import type { Profile } from "./types.js";
import type { QueryClient } from "@tanstack/react-query";

const profilesCollection = collectionOptions("profiles",
    (client) => queryCollectionOptions({
        id: "profiles",
        queryKey: ["profiles"],
        queryClient: client.requireDependency<QueryClient>("queryClient"),
        queryFn: async () => {
            const time = Temporal.Now.plainDateTimeISO().toString();

            return [
                {
                    name: "Example",
                    id: "test_example",
                    icon: "leaf",
                    mods: [
                        { version: "10.0.0", name: "BepinEx", author: "BepinEx", enabled: true, icon: "somepath", id: "adeddd" }
                    ],
                    dir: "C:\\Repos",
                    launch_args: "",
                    created_at: time,
                    updated_at: time,
                } as Profile
            ] as Profile[];
        },
        getKey: item => item.id,
        onInsert: async ({ transaction, collection }) => {
            collection.utils.writeBatch(() => {
                for (const m of transaction.mutations) collection.utils.writeInsert(m.modified);
            });

            return { refetch: false }
        },
        onUpdate: async ({ transaction, collection }) => {
            //TODO: use ipc to notify host about profiles state change

            collection.utils.writeBatch(() => {
                for (const m of transaction.mutations) collection.utils.writeUpdate(m.modified);
            });

            return { refetch: false }
        },
        onDelete: async ({ transaction, collection }) => {
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

    const mods = query?.length ? data?.mods.filter(e => e.name.includes(query)) : data?.mods;

    return { data: mods, isLoading, isError, persistedError };
}