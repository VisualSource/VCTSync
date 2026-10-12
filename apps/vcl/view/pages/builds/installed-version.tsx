import { invoke } from "react-iced-native";
import { useQuery, queryOptions } from "@tanstack/react-query";
import Network from "../../assets/icons/network.svg" with { type: "svg" };
import type { Version } from "./types.js";

export const InstalledVersion = () => {
    const { isLoading, isError, error, data } = useQuery(queryOptions({
        queryKey: ["builds::active"],
        queryFn: async () => {
            const data = await invoke<Version | null>("fetch::install_build", {});
            return data;
        }
    }),);

    if (isLoading) return (
        <col>
            <text>Loading</text>
        </col>
    );


    if (isError) return (
        <col>
            <text>{error.message}</text>
        </col>
    );


    if (!data) return (
        <row>
            <text>Installed:</text>
            <space width={6} />
            <text>No Installed Version</text>
        </row>
    );

    return (
        <row>
            <text>Installed:</text>

            <space width={6} />

            <row>
                <view width="shrink">
                    <svg src={Network} width={18} height={18} />
                </view>
                <view padding={[4, 8]} style="roundedBox">
                    <text>
                        {data.version}
                    </text>
                </view>
            </row>
        </row>
    );
}