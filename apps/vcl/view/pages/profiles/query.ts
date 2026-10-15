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
            return [
                {
                    name: "Example",
                    id: "test_example",
                    icon: "leaf",
                    mods: [
                        { name: "BepinEx", author: "BepinEx", enabled: true, icon: "somepath", id: "adeddd" }
                    ],
                    dir: "C:\\Repos",
                    launch_args: ""
                }
            ] as Profile[];
        },
        getKey: item => item.id,
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

export const useModList = (profileId: string | null) => {
    const { data, ...rest } = useProfile(profileId);
    return { data: data?.mods, ...rest };
}