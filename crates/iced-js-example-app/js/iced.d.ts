declare module "iced" {
    export class Font {
        constructor();
    }
    export class Color {
        constructor();
    }

    export class LineHeight {
        constructor(type: "relative", value: number);
        constructor(type: "absolute", value: IcedParams.Pixels);
    }
}