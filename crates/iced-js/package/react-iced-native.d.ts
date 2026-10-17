/// <reference types="react"/>
/// <reference path="./elements.d.ts"/>


declare class SvgHandle {
    private constructor();
}

declare module "*.svg" {
    const handle: SvgHandle;
    export default handle;
}

declare module "iced:compression" {
    export function gzipDecodeSync(data: Uint8Array): Uint8Array;
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

    /**
     * An `<anim>` whose contents sit in an `<Activity>`: once it has animated
     * back to `from` the contents are hidden and their effects cleaned up, and
     * they become visible again as soon as `active` turns back on.
     */
    export function AnimActivity(props: Omit<Iced.Elements.Anim, "hideWhenInactive">): React.ReactNode;
}
