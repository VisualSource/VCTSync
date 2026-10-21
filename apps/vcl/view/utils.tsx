import { Children } from "react";
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

/** Centers content both ways in the available space. A `col` can't do this itself: iced columns
 * only align children horizontally, so `alignY` on a col has no effect. */
export const Center = ({ children, spacing }: React.PropsWithChildren<{ spacing?: number }>) => (
    <view center="fill">
        <col alignX="center" spacing={spacing ?? 0}>
            {Children.toArray(children)}
        </col>
    </view>
);
