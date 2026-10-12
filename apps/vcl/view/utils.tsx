export const Tooltip = ({ children, tip, position = "right" }: React.PropsWithChildren<{ position?: Iced.TooltipPosition; tip: string }>) => {
    return (
        <tooltip position={position}>
            {children}
            <view padding={[4, 8]} style="roundedBox">
                <text>{tip}</text>
            </view>
        </tooltip>
    );
}