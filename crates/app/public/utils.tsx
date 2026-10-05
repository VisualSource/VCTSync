export const Tooltip = ({ children, tip, position = "right" }: React.PropsWithChildren<{ position?: string; tip: string }>) => {
    return (
        <tooltip position={position}>
            <view padding={[4, 8]}>
                <text>{tip}</text>
            </view>
            {children}
        </tooltip>
    );
}