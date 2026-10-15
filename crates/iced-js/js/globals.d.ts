
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
/** Installed by `js_host::url`. */
declare class URL {
    constructor(url: string | URL, base?: string | URL);
    static canParse(url: string | URL, base?: string | URL): boolean;
    static parse(url: string | URL, base?: string | URL): URL | null;
    href: string;
    readonly origin: string;
    protocol: string;
    username: string;
    password: string;
    host: string;
    hostname: string;
    port: string;
    pathname: string;
    search: string;
    hash: string;
    readonly searchParams: URLSearchParams;
    toString(): string;
    toJSON(): string;
}

declare class URLSearchParams {
    constructor(init?: string[][] | Record<string, string> | string | URLSearchParams);
    readonly size: number;
    append(name: string, value: string): void;
    delete(name: string, value?: string): void;
    get(name: string): string | null;
    getAll(name: string): string[];
    has(name: string, value?: string): boolean;
    set(name: string, value: string): void;
    sort(): void;
    forEach(callback: (value: string, key: string, parent: URLSearchParams) => void, thisArg?: any): void;
    entries(): IterableIterator<[string, string]>;
    keys(): IterableIterator<string>;
    values(): IterableIterator<string>;
    [Symbol.iterator](): IterableIterator<[string, string]>;
    toString(): string;
}
