import { useQuery } from "@tanstack/react-query";
import LayersPlus from "../assets/icons/layers-plus.svg" with { type: "svg" };

type Profile = {
    name: string;
    id: string;
    mods: {
        name: string;
        icon: string;
        id: string;
    }[]
}

export const Profiles = () => {
    const { isLoading, isError, error, data } = useQuery({
        queryKey: ["profiles"],
        queryFn: async () => {
            return [] as Profile[];
        }
    });

    return (
        <col>
            <row>
                <text>Active Profile: None</text>
                <space width="fill" />
                <button onPress={() => { }}>Launch</button>
            </row>
            <hr />
            <space height={10} />
            <View isError={isError} isLoading={isLoading} error={error} data={data} />
        </col>
    )
}

const View = ({ isLoading, isError, error, data }: { isLoading: boolean; isError: boolean; error: Error | null, data: Profile[] | undefined }) => {
    if (isLoading) {
        return (
            <view height="fill" center="fill" width="fill">
                <col alignX="center" spacing={6}>
                    <text>Loading</text>
                </col>
            </view>
        );
    }

    if (isError) {
        return (
            <view height="fill" center="fill" width="fill">
                <col alignX="center" spacing={6}>
                    <text>{error?.message ?? String(error)}</text>
                </col>
            </view>
        );
    }

    if (!data) {
        return (
            <view height="fill" center="fill" width="fill">
                <col alignX="center" spacing={6}>
                    <text>There was an issue loading profiles try again</text>
                </col>
            </view>
        );
    }

    if (data.length === 0) {
        return (
            <view height="fill" center="fill" width="fill">
                <col alignX="center" spacing={6}>
                    <text center>No profiles</text>
                    <button onPress={() => { }}>
                        <row width="shrink" spacing={6}>
                            <text>Create</text>
                            <svg src={LayersPlus} />
                        </row>
                    </button>
                </col>
            </view>
        );
    }

    return (
        <scroll>
            {data.map(profile => (
                <view key={profile.id}>
                    <row>
                        <text>{profile.name}</text>
                        <space />



                    </row>
                </view>
            ))}
        </scroll>
    );
}