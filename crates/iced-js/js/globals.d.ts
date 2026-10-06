
declare function setTimeout(fn: () => void, ms?: number): number;
declare function clearTimeout(id: number): void;
declare function queueMicrotask(cb: () => void): void;

declare class SvgHandle {
    private constructor();
}

/** Installed by `js_host::console`, not the DOM — `lib` is ES2020 only. */
declare const console: {
    debug(...values: unknown[]): void;
    log(...values: unknown[]): void;
    warn(...values: unknown[]): void;
    error(...values: unknown[]): void;
};

type BufferSource = ArrayBufferView<ArrayBuffer> | ArrayBuffer;
declare const crypto: {
    getRandomValues<T extends Exclude<BufferSource, ArrayBuffer>>(array: T): T;
}