import { queryOptions, useQuery } from "@tanstack/react-query";
import { BuildsList } from "./builds-list.js";
import type { Version } from "./types.js";

type Props = {
    icon: SvgHandle
    name: string;
    queryKey: string;
    queryFn: (args: { signal: AbortSignal }) => Promise<Version[]>
}

export const BuildGroup = ({ name, queryKey, icon, queryFn }: Props) => {
    const { isLoading, isError, error, data, refetch } = useQuery(queryOptions({
        queryKey: [queryKey],
        queryFn
    }));

    return (
        <col spacing={4}>
            <row padding={2} alignY="center">
                <text>{name}</text>
                <space width="fill" />
                <button disabled={isLoading} onPress={() => void refetch().catch(err => console.error(err))}>
                    Refetch
                </button>
            </row>
            <hr />

            <row height={256}>
                <scroll spacing={4}>
                    <BuildsList
                        isError={isError}
                        error={error}
                        isLoading={isLoading}
                        data={data}
                        icon={icon} />
                </scroll>
            </row>
        </col>
    );
}