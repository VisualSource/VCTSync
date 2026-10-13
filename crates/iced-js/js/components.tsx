import { Activity, useState } from "react";

type AnimActivityProps = Omit<Iced.Elements.Anim, "hideWhenInactive" | "onSettle">;

/**
 * An `<anim>` whose contents sit in an `<Activity>`: once it has animated
 * back to `from` the contents are hidden and their effects cleaned up, and
 * they become visible again as soon as `active` turns back on.
 */
export const AnimActivity = ({ active, children, ...anim }: AnimActivityProps) => {
    const [hidden, setHidden] = useState(!active);
    // Show the contents in the same render that starts the animation in.
    if (active && hidden) setHidden(false);

    return (
        <anim
            {...anim}
            active={active}
            hideWhenInactive
            onSettle={(ev) => {
                if (!ev.value) setHidden(true);
            }}
        >
            <Activity mode={hidden ? "hidden" : "visible"}>{children}</Activity>
        </anim>
    );
};
