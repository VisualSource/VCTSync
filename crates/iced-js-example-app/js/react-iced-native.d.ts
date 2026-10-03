declare global {
    var __ICED_INTERNALS__: {
        commentTree(rootId: string, tree: IcedChild): void;
        // object needs to json stringible
        invoke(cmd: string, obj: object): void;
        addEventListener(type: string, cb: (ev: unknown) => void, opts?: { once: boolean }): void;
        removeEventListener(type: string, cb: (ev: unknown) => void): void;
    }
}

declare class SvgHandle {
    private constructor();
}

declare module "*.svg" {
    declare const handle: SvgHandle;
    export default handle;
}

declare module "react-iced-native" {

    export function createRoot(id: string): { render: (el: React.ReactNode) => void, destroy(): void; }
}



declare module "react/jsx-runtime" {

    namespace Iced {
        type Length = "fill" | "shrink" | number | `${number}%`;
        type Horizontal = "center" | "left" | "right";
        type Vertical = "center" | "bottom" | "top";
        type Padding = [number, number] | number;
        type Pixels = number;

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
            }

            interface Svg extends RecordingState.Attributes {
                width?: Pixels;
                height?: Pixels;

                src: SvgHandle
            }

            interface Col extends Common { }
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
                children: string
            }
            interface Button extends React.Attributes {
                onPress?: () => void;
                clip?: boolean;
                padding?: Padding;
                width?: Length,
                height?: Length,

                children: React.ReactNode;
            }
            interface View extends React.Attributes {
                id?: string;
                padding?: Padding;
                width?: Length,
                height?: Length,

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
            interface Scroll extends React.Attributes {
                width?: IcedParams.Length;

                width?: Length;
                height?: Length;
                horizontal?: boolean;

                [`anchor${"Bottom" | "Left" | "Top" | "Right"}`]?: boolean;
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
                sanpWithinViewport?: boolean;
                /**
                 * @default "top"
                 */
                position?: "top" | "bottom" | "left" | "right" | "followCursor"

                /**
                 * 
                 */
                children: [React.ReactNode, React.ReactNode]
            }
            interface Float extends React.Attributes {
                scale?: number;
            }

            interface Textarea extends React.Attributes {
                children?: string
            }

            interface Hr extends React.Attributes {
                /** @default 1 */
                height?: Pixels;
            }
            interface Vr extends React.Attributes {
                /** @default 1 */
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
                }
            }

            type Input = (Inputs.Radio | Inputs.Checkbox | Inputs.Text | Inputs.Switch | Inputs.Range) & React.Attributes;
        }
    }

    namespace JSX {
        interface IntrinsicElements {
            col: Iced.Elements.Col,
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