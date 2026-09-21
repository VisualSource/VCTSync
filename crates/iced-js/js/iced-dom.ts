import { createContext } from "react";
import * as ReactNamespace from "react";
import * as JsxRuntimeNamespace from "react/jsx-runtime";
import Reconciler, { type HostConfig, type ReactContext, type EventPriority, type OpaqueRoot } from "react-reconciler";
import { ConcurrentRoot, DefaultEventPriority, NoEventPriority } from "react-reconciler/constants";

/**
 * React and its JSX runtime, re-exported so that `vendor/react.ts` and
 * `vendor/jsx-runtime.ts` can forward *this* instance rather than bundling
 * their own.
 *
 * React has to be bundled into this module rather than left external:
 * `react-reconciler` ships as CommonJS and calls `require("react")` at
 * runtime, which esbuild can only turn into a dynamic require that throws.
 * So this module owns the single copy, and the `react` / `react/jsx-runtime`
 * specifiers that user scripts import resolve to thin façades over these two
 * bindings. One instance means the reconciler's hook dispatcher is the same
 * one a user component's `useState` reaches for.
 */
export const React = ReactNamespace;
export const jsxRuntime = JsxRuntimeNamespace;

declare function setTimeout(fn: () => void, ms?: number): number;
declare function clearTimeout(id: number): void;
declare function queueMicrotask(cb: () => void): void;


declare global {
    /** The host object Rust installs. */
    var iced: {
        comment_tree(rootId: string, tree: IcedChild): void;
    }

    /** This module's entry points, published as a global for Rust to call
     *  directly. Reaching them through an `import` instead would mean a
     *  module-type eval per call, and every one of those registers a module
     *  def that is never freed until the context dies. */
    var iced_runtime: {
        destroyRoot(id: string): void;
        dispatch(id: number, payload?: unknown): void;
    }
}

/** Installed by `js_host::console`, not the DOM — `lib` is ES2020 only. */
declare const console: {
    debug(...values: unknown[]): void;
    log(...values: unknown[]): void;
    warn(...values: unknown[]): void;
    error(...values: unknown[]): void;
};


type IcedTag = string;
/** Props as React hands them to us: anything at all, including functions,
 *  `undefined` and the `children` React folds in. */
type RawProps = Record<string, unknown>;
/** Props after {@link sanitizeProps}: only what the Rust side can decode.
 *  Callbacks have already been swapped for their registry id. */
type IcedProps = Record<string, string | number | boolean>;
type IcedNode = { type: IcedTag, props: IcedProps, children: IcedChild[] };
type IcedText = { text: string }
type IcedChild = IcedNode | IcedText;
type IcedContainer = { commit(children: readonly IcedChild[]): void }


type IcedHostConfig = HostConfig<IcedTag, RawProps, IcedContainer, IcedNode, IcedText, never, never, never, IcedNode, null, IcedChild[], number, -1, null>;

/* -------------------------------------------------------------------------
 * Callback registry
 *
 * Functions cannot cross into Rust, so every function prop is replaced by a
 * numeric id and kept here. Rust sends the id back through `dispatch`.
 *
 * Ids churn: persistent mode rebuilds every instance along a changed path, so
 * each commit mints fresh ids for those nodes. The registry is therefore
 * pruned to the ids actually present in the committed tree — see `commit`.
 * ---------------------------------------------------------------------- */

const callbacks = new Map<number, Function>();
let nextCallbackId = 1;

const registerCallback = (fn: Function): number => {
    const id = nextCallbackId++;
    callbacks.set(id, fn);
    return id;
};

/**
 * Raise the id counter past everything a previous context minted.
 *
 * A rebuilt context gets a fresh copy of this module, so ids would otherwise
 * restart at 1 and a dispatch still in flight from the old tree would land on
 * an unrelated handler. Staying monotonic keeps `dispatch`'s miss the no-op it
 * is documented to be.
 */
export const setCallbackBase = (n: number): void => {
    if (n > nextCallbackId) nextCallbackId = n;
};

/** Collect every callback id reachable from `node` into `live`. */
const collectCallbackIds = (node: IcedChild, live: Set<number>): void => {
    if ("text" in node) return;

    for (const key in node.props) {
        if (isCallbackProp(key)) {
            const id = node.props[key];
            if (typeof id === "number") live.add(id);
        }
    }

    for (const child of node.children) collectCallbackIds(child, live);
};

/** A prop holding a callback id rather than a plain value. Keeping the naming
 *  convention in one place means Rust and TS can't drift apart on it. */
const isCallbackProp = (key: string): boolean => key.startsWith("on");

/**
 * Turn React's props into something `to_node` on the Rust side can decode.
 *
 * - `children` is dropped: React folds children into props, but the reconciler
 *   also delivers them through `appendInitialChild`, so keeping them would
 *   duplicate the subtree and hand Rust an object it would skip anyway.
 * - Functions become callback ids.
 * - `undefined` and `null` are dropped. React passes them constantly for
 *   optional props, and the Rust prop loop has no representation for either.
 * - Anything else non-primitive is dropped with a warning, matching what Rust
 *   would do with it, but reported on the side that knows the component.
 */
const sanitizeProps = (type: IcedTag, props: RawProps): IcedProps => {
    const out: IcedProps = {};

    for (const key in props) {
        if (key === "children") continue;

        const value = props[key];

        if (value === undefined || value === null) continue;

        switch (typeof value) {
            case "string":
            case "number":
            case "boolean":
                out[key] = value;
                break;
            case "function":
                if (!isCallbackProp(key)) {
                    console.warn(`<${type}> prop "${key}" is a function but is not named on*; ignoring it`);
                    continue;
                }
                out[key] = registerCallback(value as Function);
                break;
            default:
                console.warn(`<${type}> prop "${key}" has unsupported type ${typeof value}; ignoring it`);
                break;
        }
    }

    return out;
};

let currentPriority: EventPriority = NoEventPriority;
const config: IcedHostConfig = {
    supportsMutation: false,
    supportsHydration: false,
    supportsMicrotasks: true,
    supportsPersistence: true,


    /** Persistent Mode: main hooks  */
    createInstance(type, props, rootContainer, hostContext, internalHandle) {
        return {
            type,
            props: sanitizeProps(type, props),
            children: []
        }
    },
    createTextInstance(text, rootContainer, hostContext, internalHandle) {
        return {
            text
        }
    },
    appendInitialChild(parentInstance, child) {
        parentInstance.children.push(child);
    },
    cloneInstance(instance, type, oldProps, newProps, keepChildren, recyclableInstance) {
        // `newProps`, not `instance.props` — this is the only path by which a
        // prop change reaches the committed tree.
        return {
            type,
            props: sanitizeProps(type, newProps),
            children: keepChildren ? instance.children : []
        }
    },
    createContainerChildSet(container) {
        return [];
    },
    appendChildToContainerChildSet(childSet, child) {
        childSet.push(child);
    },
    finalizeContainerChildren(container, newChildren) {
        // Nothing to freeze: the child set is a plain array handed straight to
        // `replaceContainerChildren`.
    },
    replaceContainerChildren(container, newChildren) {
        container.commit(newChildren);
    },
    clearContainer(container) {
        container.commit([]);
    },
    cloneHiddenInstance(instance, type, props, internalInstanceHandle) {

        return {
            type,
            props: sanitizeProps(type, props),
            children: instance.children
        }
    },
    cloneHiddenTextInstance(instance, text, internalInstanceHandle) {

        return {
            text
        }
    },
    shouldSetTextContent(type, props) {
        return false;
    },
    getPublicInstance(instance) {
        return instance as IcedNode;
    },

    /** Context/commit hooks  */
    getRootHostContext() {
        return null;
    },
    getChildHostContext(parentHostContext, type, rootContainer) {
        return null;
    },
    finalizeInitialChildren(instance, type, props, rootContainer, hostContext) {
        return false;
    },
    prepareForCommit() {
        return null;
    },
    resetAfterCommit(containerInfo) {
        // Called after every commit. There is no host state to restore — the
        // tree has already gone to Rust from `replaceContainerChildren`.
    },
    preparePortalMount(containerInfo) {
        // Portals are not supported; nothing to prepare.
    },
    isPrimaryRenderer: true,


    /**Timers */
    scheduleTimeout: setTimeout,
    cancelTimeout: clearTimeout,
    noTimeout: -1,
    scheduleMicrotask: queueMicrotask,


    /**Update Priority */
    setCurrentUpdatePriority(newPriority) {
        currentPriority = newPriority;
    },
    getCurrentUpdatePriority() {
        return currentPriority;
    },
    resolveUpdatePriority() {
        // Inside an event React has already set a priority; outside one there
        // is no host event to derive it from, so fall back to the default.
        return currentPriority !== NoEventPriority ? currentPriority : DefaultEventPriority;
    },

    HostTransitionContext: createContext(null) as unknown as ReactContext<null>,
    NotPendingTransition: null,


    /**Other */
    getInstanceFromNode(node) {
        return null;
    },
    beforeActiveInstanceBlur() {
        // No focus model on this host.
    },
    afterActiveInstanceBlur() {
        // No focus model on this host.
    },
    prepareScopeUpdate(scopeInstance, instance) {
        // Scopes are unused.
    },
    getInstanceFromScope(scopeInstance) {
        return null;
    },
    detachDeletedInstance(node) {
        // Instances hold no host resources; the callback registry is pruned on
        // commit instead, since persistent mode discards clones without
        // routing them all through here.
    },
    resetFormInstance(form) {
        // No form instances on this host.
    },
    requestPostPaintCallback(callback) {
        // There is no paint to hook; iced redraws on its own schedule.
    },
    shouldAttemptEagerTransition() {
        return false;
    },
    trackSchedulerEvent() {
        // No scheduler profiling.
    },
    resolveEventType() {
        return null;
    },
    resolveEventTimeStamp() {
        return -1.1;
    },
    maySuspendCommit(type, props) {
        return false;
    },
    preloadInstance(type, props) {
        return true;
    },
    startSuspendingCommit() {
        // `maySuspendCommit` is always false, so this is never reached.
    },
    suspendInstance(type, props) {
        // `maySuspendCommit` is always false, so this is never reached.
    },
    waitForCommitToBeReady() {
        return null;
    },

};

/** Each root's last committed tree is kept so callback pruning can see every
 *  root's live ids, not just the one currently committing. */
const roots = new Map<string, { root: OpaqueRoot, tree?: IcedChild }>();
const reconciler = Reconciler(config);
const onError = (err: Error) => { console.error(err); }

/** An empty root. `col` with no children renders as an empty `Column`. */
const emptyNode = (): IcedNode => ({ type: "col", props: {}, children: [] });

export const createRoot = (id: string) => {
    const container: IcedContainer = {
        commit(children) {
            // Rust takes a single root node, so collapse the child set:
            // nothing renders as an empty column, and a fragment at the root
            // is wrapped in one.
            let tree: IcedChild;
            if (children.length === 0) {
                tree = emptyNode();
            } else if (children.length === 1) {
                tree = children[0]!;
            } else {
                tree = { type: "col", props: {}, children: [...children] };
            }

            // Drop callbacks that no longer appear in any root's committed
            // tree. Their instances were replaced by clones during this render,
            // so nothing can dispatch to them again. The registry is shared
            // across roots, so the sweep has to consider all of them or this
            // commit would delete a sibling root's live ids.
            const entry = roots.get(id);
            if (entry) entry.tree = tree;

            const live = new Set<number>();
            for (const other of roots.values()) {
                if (other.tree) collectCallbackIds(other.tree, live);
            }
            for (const callbackId of callbacks.keys()) {
                if (!live.has(callbackId)) callbacks.delete(callbackId);
            }

            iced.comment_tree(id, tree);
        },
    }

    const root = reconciler.createContainer(container, ConcurrentRoot, null, false, null, "", onError, onError, onError, () => { });

    roots.set(id, { root, });

    return {
        destroy() {
            destroyRoot(id);
        },
        render: (children: React.ReactNode) => {
            reconciler.updateContainer(children, root, null, null)
        }
    }
}

/** Tear a root down from Rust (`JsCmd::Unmount`). Unmounting renders `null`,
 *  which commits an empty tree and prunes that root's callbacks. */
export const destroyRoot = (id: string): void => {
    const entry = roots.get(id);
    if (!entry) return;

    reconciler.updateContainer(null, entry.root, null, null);
    roots.delete(id);
};

/** Invoke a callback by id. Called from Rust when an iced widget fires
 *  (`JsCmd::Dispatch`). An id with no entry is a stale event from a tree that
 *  has since been replaced, which is expected rather than an error. */
export const dispatch = (id: number, payload?: unknown): void => {
    const fn = callbacks.get(id);
    if (!fn) return;

    try {
        fn(payload);
    } catch (err) {
        console.error(err);
    }
};


globalThis.iced_runtime = {
    destroyRoot,
    dispatch,
}