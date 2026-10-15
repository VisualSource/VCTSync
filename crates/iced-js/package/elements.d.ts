/// <reference types="react"/>

// Element typings, kept apart from the module declaration so iced-js itself
// can include them without shadowing its own `react-iced-native` module.
// Expects `SvgHandle` to be declared alongside.

declare namespace Iced {
    type Length = "fill" | "shrink" | number | `${number}%`;
    type Horizontal = "center" | "left" | "right";
    type Vertical = "center" | "bottom" | "top";
    type Padding = [number, number, number, number] | [number, number, number] | [number, number] | number;
    type Pixels = number;
    type TooltipPosition = "top" | "bottom" | "left" | "right" | "followCursor";
    type Font = {
        style?: unknown
        weight?: "extraLight" |
        "light" |
        "thin" |
        "normal" |
        "medium" |
        "semibold" |
        "bold" |
        "extraBold" |
        "black"
        family?: string;
        stretch?: "UltraCondensed" | "ExtraCondensed" | "Condensed" | "SemiCondensed" | "Normal" | "SemiExpanded" | "Expanded" | "ExtraExpanded" | "UltraExpanded"
    } | "default" | "monospace";
    namespace Elements {
        interface Stack extends React.Attributes {
            height?: Length,
            width?: Length,
            clip?: boolean;

            children?: React.ReactNode[]
        }

        interface Common extends React.Attributes {
            children?: React.ReactNode[];
            padding?: Padding;
            height?: Length;
            width?: Length;
            alignX?: Horizontal;
            alignY?: Vertical;
            clip?: boolean;
            warp?: boolean;

            spacing?: Pixels;
        }

        interface Select extends React.Attributes {
            value?: string | undefined,
            options?: (string | { value: string; label: string })[];
            placeholder?: string;
            onSelect: (ev: { type: "change", value: string; }) => void;
        }

        interface Svg extends React.Attributes {
            width?: Pixels;
            height?: Pixels;
            color?: ThemeColor;
            /** 0–1. */
            opacity?: number;
            src: SvgHandle;
        }

        interface Column extends Common { }
        interface Row extends Common { }
        interface Text extends React.Attributes {
            center?: boolean;
            width?: Length;
            height?: Length;
            alignX?: Horizontal;
            alignY?: Vertical;

            wrapping?: boolean;

            shaping?: "auto" | "basic" | "advanced"
            children: string;
            size?: Pixels;

            color?: ThemeColor,

            font?: Font
        }
        interface Button extends React.Attributes {
            onPress?: () => void;
            clip?: boolean;
            padding?: Padding;
            width?: Length,
            height?: Length,

            disabled?: boolean;

            children: React.ReactNode;

            style?: "text" |
            "background" |
            "success" |
            "danger" |
            "primary" |
            "secondary" |
            "warning" |
            "subtle" | ButtonStyle
        }
        interface ViewStyle {
            background?: ThemeColor;
            color?: ThemeColor;
            /** Multiplies the alpha of the style's colors (background, text, border, shadow), 0–1. Unlike CSS, children with their own colors are not faded. */
            opacity?: number;
            borderWidth?: number | undefined;
            borderColor?: ThemeColor | undefined;
            borderRadius?: number | string;
            shadowColor?: ThemeColor;
            shadowBlurRadius?: number;
            shadowOffsetX?: number;
            shadowOffsetY?: number;
            snap?: boolean;
        }
        /** Unprefixed keys are the active style; `hover*`, `pressed*` and `disabled*` keys override it for that status. */
        type ButtonStyle = ViewStyle & {
            [K in keyof ViewStyle as `${"hover" | "pressed" | "disabled"}${Capitalize<K>}`]?: ViewStyle[K];
        }
        type PaletteShade = "base" | "weak" | "strong";
        type BackgroundShade = PaletteShade | "weakest" | "weaker" | "neutral" | "stronger" | "strongest";
        type PaletteToken =
            | "background" | `background.${BackgroundShade}` | "background.text" | `background.${BackgroundShade}.text`
            | `${"primary" | "secondary" | "success" | "warning" | "danger"}${"" | `.${PaletteShade}`}${"" | ".text"}`;
        /** A color string (hex/rgb) or a theme palette token resolved at draw time. */
        type ThemeColor = PaletteToken | (string & {});
        interface View extends React.Attributes {
            id?: string;
            padding?: Padding;
            width?: Length,
            height?: Length,

            style?: "roundedBox" | ViewStyle

            center?: Length

            maxWidth?: Pixels;
            maxHeight?: Pixels;
            centerX?: Length;
            centerY?: Length;
            alignLeft?: Length;
            alignRight?: Length;
            alignTop?: Length;
            alignBottom?: Length;
            alignX?: Horizontal;
            alignY?: Vertical;
            clip?: boolean;

            children: React.ReactNode;
        }

        type ScrollAnchorNames = `anchor${"Bottom" | "Left" | "Top" | "Right"}`;

        type ScrollAnchors = {
            [Prop in ScrollAnchorNames]?: boolean;
        }

        interface Scroll extends React.Attributes, ScrollAnchors {
            width?: Length;
            height?: Length;
            horizontal?: boolean;
            autoScroll?: boolean;
            spacing?: Pixels;
            direction?: unknown;
            onScroll?: (viewport: unknown) => void
            id?: string;

            anchorX?: unknown;
            anchorY?: unknown;
            style?: unknown;

            children: React.ReactNode;
        }
        interface Space extends React.Attributes {
            width?: Length;
            height?: Length;
        }
        interface Tooltip extends React.Attributes {
            padding?: Pixels;
            gap?: Pixels;
            snapWithinViewport?: boolean;
            /** @default "top" */
            position?: TooltipPosition;


            children: [React.ReactNode, React.ReactNode]
        }
        interface Float extends React.Attributes {
            scale?: number;
        }

        /** A transform applied to the contents of an `anim`. Unset fields are the identity. */
        interface Keyframe {
            /** Pixels, or a percentage of the element's own width. */
            x?: number | `${number}%`;
            /** Pixels, or a percentage of the element's own height. */
            y?: number | `${number}%`;
            scale?: number;
        }

        type Easing = "linear" | "easeIn" | "easeOut" | "easeInOut" | "easeInCubic" | "easeOutCubic" | "easeInOutCubic" | "easeOutBack" | "easeOutBounce";

        interface Anim extends React.Attributes {
            /** Tweens to `to` when true, back to `from` when false. */
            active: boolean;
            from?: Keyframe;
            to?: Keyframe;
            /** In milliseconds. @default 200 */
            duration?: number;
            /** @default "easeOut" */
            easing?: Easing;
            /** Skip drawing and input once fully back at `from`. @default false */
            hideWhenInactive?: boolean;
            /** Fires once the animation comes to rest, with the state it ended on. */
            onSettle?: (ev: { type: "change", value: boolean }) => void;

            children: React.ReactNode;
        }

        interface Textarea extends React.Attributes {
            children?: string
        }

        interface Hr extends React.Attributes {
            /** @default 1*/
            height?: Pixels;
        }
        interface Vr extends React.Attributes {
            /** @default 1*/
            width?: Pixels
        }

        interface Opaque extends React.Attributes {
            children: React.ReactNode;
        }


        namespace Inputs {
            interface Range {
                type: "range"
            }
            interface Text {
                type: "text",
                value: string;
                onSubmit?: (ev: { type: "submit" }) => void;
                placeholder?: string;
                onChange?: (ev: { type: "change", value: string }) => void;
                size?: Pixels;
                padding?: Padding;
            }
            interface Switch {
                type: "switch",
                checked?: boolean;
                onChange?: (ev: { type: "change", value: boolean }) => void;
                label?: string;
                disabled?: boolean;
                width?: Length;
                spacing?: Pixels;
                size?: Pixels;
                font?: Font;
                textSize?: Pixels;
            }
            interface Radio {
                type: "radio"
            }
            interface Checkbox {
                type: "checkbox"
                checked: boolean;
                label?: string;
                onChange: (ev: { type: "change", value: boolean }) => void
            }
        }

        type Input = (Inputs.Radio | Inputs.Checkbox | Inputs.Text | Inputs.Switch | Inputs.Range) & React.Attributes;
    }

}

declare module "react/jsx-runtime" {
    // This is an ambient declaration, so it replaces React's typings for the
    // module (dropping the DOM elements) instead of merging with them, and
    // has to restate the runtime exports.
    export const Fragment: typeof import("react").Fragment;
    export function jsx(type: React.ElementType, props: unknown, key?: React.Key): React.ReactElement;
    export function jsxs(type: React.ElementType, props: unknown, key?: React.Key): React.ReactElement;

    namespace JSX {
        interface IntrinsicElements {
            col: Iced.Elements.Column,
            row: Iced.Elements.Row,
            view: Iced.Elements.View,
            button: Iced.Elements.Button,
            text: Iced.Elements.Text,
            scroll: Iced.Elements.Scroll,
            space: Iced.Elements.Space,
            hr: Iced.Elements.Hr,
            vr: Iced.Elements.Vr,
            tooltip: Iced.Elements.Tooltip,
            /** A widget that can make its contents float over other widgets. */
            float: Iced.Elements.Float,
            /** Animates its contents between two keyframes when `active` changes. */
            anim: Iced.Elements.Anim,

            input: Iced.Elements.Input,

            // only when svg-element is enabled
            svg: Iced.Elements.Svg,

            textarea: Iced.Elements.Textarea;
            select: Iced.Elements.Select;

            /** container that displays children on top of each other. */
            stack: Iced.Elements.Stack,
            /** 
             * Wraps the given widget and captures any mouse button presses inside the bounds of the widget—effectively making it opaque.
             *
             * This helper is meant to be used to mark elements in a Stack to avoid mouse events from passing through layers.
            */
            opaque: Iced.Elements.Opaque
        }
    }
}
