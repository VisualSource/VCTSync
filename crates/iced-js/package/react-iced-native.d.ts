/// <reference types="react"/>


declare class SvgHandle {
    private constructor();
}

declare module "*.svg" {
    const handle: SvgHandle;
    export default handle;
}

declare module "react-iced-native" {

    /**
     * 
     *
     * @export
     * @template T
     * @param {string} cmd
     * @param {object} obj should be json stringifyable
     * @return {*}  {Promise<T>}
     */
    export function invoke<T>(cmd: string, obj: unknown): Promise<T>;


    export function createRoot(id: string): { render: (el: React.ReactNode) => void, destroy(): void; }
}

declare namespace Iced {
    type Length = "fill" | "shrink" | number | `${number}%`;
    type Horizontal = "center" | "left" | "right";
    type Vertical = "center" | "bottom" | "top";
    type Padding = [number, number] | number;
    type Pixels = number;
    type TooltipPosition = "top" | "bottom" | "left" | "right" | "followCursor";

    namespace Elements {
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

        interface Svg extends React.Attributes {
            width?: Pixels;
            height?: Pixels;

            src: SvgHandle
        }

        interface Column extends Common { }
        interface Row extends Common { }
        interface Text extends React.Attributes {

            center?: boolean;

            onLinkClick?: () => void;
            width?: Length;
            height?: Length;
            alignX?: Horizontal;
            alignY?: Vertical;

            wrapping?: boolean;

            shaping?: "auto" | "basic" | "advanced"
            children: string;
            size?: Pixels;

            font?: {
                style?: unknown
                weight?: "bold" | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900;
                family?: string;
                stretch?: "UltraCondensed" | "ExtraCondensed" | "Condensed" | "SemiCondensed" | "Normal" | "SemiExpanded" | "Expanded" | "ExtraExpanded" | "UltraExpanded"
            } | "default" | "monospace";
        }
        interface Button extends React.Attributes {
            onPress?: () => void;
            clip?: boolean;
            padding?: Padding;
            width?: Length,
            height?: Length,

            disabled?: boolean;

            children: React.ReactNode;
        }
        interface View extends React.Attributes {
            id?: string;
            padding?: Padding;
            width?: Length,
            height?: Length,

            style?: "roundedBox" | { background?: string; color?: string; border?: string; }

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


        namespace Inputs {
            interface Range {
                type: "range"
            }
            interface Text {
                type: "text",
                value: string;
                placeholder: string;
                onChange?: (ev: { type: "change", value: string }) => void
            }
            interface Switch {
                type: "switch",
                checked: boolean;
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
            float: Iced.Elements.Float,

            input: Iced.Elements.Input,

            // only when svg-element is enabled
            svg: Iced.Elements.Svg,

            textarea: Iced.Elements.Textarea
        }
    }
}