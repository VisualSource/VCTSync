type Props = { isLoading: boolean, isError: boolean, error: Error | null, data: string[] | undefined }

export const LogViewer = ({ isLoading, isError, error, data }: Props) => {
    if (isLoading) return (
        <view height="fill" center="fill" width="fill">
            <col alignX="center">
                <text>Loading</text>
            </col>
        </view>
    );


    if (isError) return (
        <view height="fill" center="fill" width="fill">
            <col alignX="center">
                <text>{error?.message ?? String(error)}</text>
            </col>
        </view>
    );


    if (!data || !data.length) return (
        <view height="fill" center="fill" width="fill">
            <col alignX="center">
                <text>No log file content</text>
            </col>
        </view>
    );

    return (
        <scroll height="fill" anchorBottom>
            <col>
                {data.map((item, i) =>
                    (<text size={12} key={i}>{item}</text>)
                )}
            </col>
        </scroll>
    );
}