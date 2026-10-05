import { useQuery } from "@tanstack/react-query";
import { invoke } from "iced";
import { useState } from "react";


export const Logs = () => {
    const [modLogLines, setModLogLines] = useState(false);
    const { isLoading, isError, error, data } = useQuery({
        queryKey: ["logs", modLogLines],
        queryFn: () => {
            return invoke<string[]>("fetch_log_lines", {});
        }
    })

    return (
        <col>
            <row>
                <input label="Mod lines only" type="checkbox" checked={modLogLines} onChange={(e) => setModLogLines(e.checked)} />
                <space width="fill" />

            </row>
            <hr />
            <Log isError={isError} isLoading={isLoading} error={error} data={data} />
        </col>
    )
}

const Log = ({ isLoading, isError, error, data }: { isLoading: boolean, isError: boolean, error: Error | null, data: string[] | undefined }) => {
    if (isLoading) {
        return (
            <view height="fill" width="fill">
                <col alignX="center" alignY="center">
                    <text>Loading</text>
                </col>
            </view>
        )
    }

    if (isError) {
        return (
            <view height="fill" width="fill">
                <col alignX="center" alignY="center">
                    <text>{error?.message ?? String(error)}</text>
                </col>
            </view>
        );
    }

    if (!data) {
        return (
            <view height="fill" width="fill">
                <col alignX="center" alignY="center">
                    <text>No log file content</text>
                </col>
            </view>
        );
    };

    return (
        <scroll height="fill" anchorBottom>
            {data.map((item, i) => {
                <text key={i}>{item}</text>
            })}
        </scroll>
    );
}