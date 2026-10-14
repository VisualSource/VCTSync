import Crown from "@icon/crown.svg" with {type: "svg"};
import Sword from "@icon/sword.svg" with {type: "svg"};
import Leaf from "@icon/leaf.svg" with {type: "svg"};
import Box from "@icon/box.svg" with {type: "svg"};
import Rocket from "@icon/rocket.svg" with {type: "svg"};
import Flask from "@icon/flask-conical.svg" with { type: "svg" };

const ICON_MAP = {
    leaf: { color: "#2f8a5b", icon: Leaf },
    flask: { color: "#7a5bd1", icon: Flask },
    rocket: { color: "#3a7bc8", icon: Rocket },
    box: { color: "#5c6370", icon: Box },
    crown: { color: "#b8892c", icon: Crown },
    sword: { color: "#b5523b", icon: Sword }
}

export type Icon = keyof typeof ICON_MAP;

type IconProps = {
    type: Icon
}

export const ProfileIcon = ({ type }: IconProps) => {
    return (
        <view alignX="center" alignY="center" width={32} height={32} style={{ background: ICON_MAP[type].color, borderRadius: 4 }}>
            <svg style="currentColor" height={15} width={15} src={ICON_MAP[type].icon} />
        </view>
    );
}

type ButtonProps = {
    onPress: () => void;
    active?: boolean;
} & IconProps;

export const IconButton = ({ active, onPress, type }: ButtonProps) => {
    return (
        <button onPress={onPress} width={32} height={32} style="background" padding={0}>
            <view alignX="center" alignY="center" width="fill" height="fill" style={{
                background: ICON_MAP[type].color,
                borderRadius: 4,
                borderColor: active ? "#ffffff" : undefined,
                borderWidth: active ? 2 : undefined
            }}>
                <svg style="currentColor" height={15} width={15} src={ICON_MAP[type].icon} />
            </view>
        </button>
    );
}