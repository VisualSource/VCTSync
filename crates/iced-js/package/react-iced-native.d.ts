declare global {
    var __ICED_INTERNALS__: {
        commentTree(rootId: string, tree: IcedChild): void;
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

declare module "iced" {
    export function invoke<T>(cmd: string, payload: object): Promise<T>;
}

declare module "react-iced-native" {
    namespace IcedParams {
        export type Length = "fill" | "shrink" | number | `${number}%`;
        export type Horizontal = "center" | "left" | "right";
        export type Vertical = "center" | "bottom" | "top";
        export type Padding = [number, number] | number;
        export type Pixels = number;
    }

    export class LineHeight {
        constructor(type: "relative", value: number);
        constructor(type: "absolute", value: IcedParams.Pixels);
    }

    export class Color {
        constructor(value: string);
    }

    export function createRoot(id: string): { render: (el: React.ReactNode) => void, destroy(): void; }
}



declare module "react/jsx-runtime" {

    namespace IcedElements {


        interface Common extends React.Attributes {
            children?: React.ReactNode[];
            padding?: IcedParams.Padding;
            height?: IcedParams.Length;
            width?: IcedParams.Length;
            alignX?: IcedParams.Horizontal;
            alignY?: IcedParams.Vertical;
            clip?: boolean;
            warp?: boolean;
        }

        interface Svg extends RecordingState.Attributes {
            width?: number;
            height?: number;

            src: SvgHandle
        }


        interface Col extends Common { }
        interface Row extends Common { }
        interface Text extends React.Attributes {

            center?: boolean;

            onLinkClick?: () => void;
            width?: IcedParams.Length;
            height?: IcedParams.Length;
            alignX?: IcedParams.Horizontal;
            alignY?: IcedParams.Vertical;

            wrapping?: boolean;

            shaping?: "auto" | "basic" | "advanced"
            children: string
        }
        interface Button extends React.Attributes {
            onPress?: () => void;
            clip?: boolean;
            padding?: IcedParams.Padding;
            width?: IcedParams.Length,
            height?: IcedParams.Length,

            children: React.ReactNode;
        }
        interface View extends React.Attributes {
            id?: string;
            padding?: IcedParams.Padding;
            width?: Length,
            height?: Length,

            maxWidth?: IcedParams.Pixels;
            maxHeight?: IcedParams.Pixels;
            centerX?: IcedParams.Length;
            centerY?: IcedParams.Length;
            alignLeft?: IcedParams.Length;
            alignRight?: IcedParams.Length;
            alignTop?: IcedParams.Length;
            alignBottom?: IcedParams.Length;
            alignX?: IcedParams.Horizontal;
            alignY?: IcedParams.Vertical;
            clip?: boolean;
            children: React.ReactNode;
        }
    }



    namespace JSX {
        interface IntrinsicElements {
            view: IcedElements.View,
            text: IcedElements.Text,
            button: IcedElements.Button,
            col: IcedElements.Col,
            row: IcedElements.Row,
            svg: IcedElements.Svg,

            scroll: React.Attributes & { children: React.ReactNode },
            hr: React.Attributes,
            vr: React.Attributes,
            space: React.Attributes
            tooltip: React.Attributes & { children: [React.ReactNode,React.ReactNode] }

            input: React.Attributes
        }
    }
}