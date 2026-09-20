import { createContext } from "react";
import Reconciler, { type HostConfig, type ReactContext, type EventPriority, type OpaqueRoot } from "react-reconciler";

declare function setTimeout(fn: () => void, ms?: number): number;
declare function clearTimeout(id: number): void;
declare function queueMicrotask(cb: () => void): void;
type IcedHost = {
    comment_tree(rootId: string, tree: IcedChild): void;
}
declare const iced: IcedHost;


type IcedTag = string;
type IcedProps = Record<string, unknown>;
type IcedNode = { type: IcedTag, props: IcedProps, children: IcedChild[] };
type IcedText = { text: string }
type IcedChild = IcedNode | IcedText;
type IcedContainer = { commit(children: readonly IcedChild[]): void }


type IcedHostConfig = HostConfig<IcedTag, IcedProps, IcedContainer, IcedNode, IcedText, never, never, never, IcedNode, null, IcedChild[], number, -1, null>;

const DefaultEventPriority: EventPriority = 0;

let currentPriority: EventPriority = -1;
const config: IcedHostConfig = {
    supportsMutation: false,
    supportsHydration: false,
    supportsMicrotasks: true,
    supportsPersistence: true,


    /** Persistent Mode: main hooks  */
    createInstance(type, props, rootContainer, hostContext, internalHandle) {
        return {
            type,
            props,
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

        return {
            type,
            props: instance.props,
            children: keepChildren ? instance.children : []
        }
    },
    createContainerChildSet(container) {
        return [];
    },
    appendChildToContainer(container, child) {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    finalizeContainerChildren(container, newChildren) {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    replaceContainerChildren(container, newChildren) {
        container.commit(newChildren);
    },
    clearContainer(container) {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    cloneHiddenInstance(instance, type, props, internalInstanceHandle) {

        return {
            type,
            props,
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
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    preparePortalMount(containerInfo) {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
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
        return DefaultEventPriority;
    },

    HostTransitionContext: createContext(null) as unknown as ReactContext<null>,
    NotPendingTransition: null,


    /**Other */
    getInstanceFromNode(node) {
        return null;
    },
    beforeActiveInstanceBlur() {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    afterActiveInstanceBlur() {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    prepareScopeUpdate(scopeInstance, instance) {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    getInstanceFromScope(scopeInstance) {
        return null;
    },
    detachDeletedInstance(node) {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    resetFormInstance(form) {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    requestPostPaintCallback(callback) {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    shouldAttemptEagerTransition() {
        return false;
    },
    trackSchedulerEvent() {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
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
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    suspendInstance(type, props) {
        throw new Error(`Unimplemented: ${arguments.callee.name}`);
    },
    waitForCommitToBeReady() {
        return null;
    },

};

const roots = new Map<string, { root: OpaqueRoot }>();
const reconciler = Reconciler(config);
const onError = (err: Error) => { console.error(err); }

export const createRoot = (id: string) => {
    const container: IcedContainer = {
        commit(children) {
            let child = children[0];
            if (!child) {
                console.error("more then one child was retuend");
                return;
            }

            iced.comment_tree(id, child);
        },
    }

    const root = reconciler.createContainer(container, 1, null, false, null, "", onError, onError, onError, () => { });

    roots.set(id, { root, });

    return {
        destory() {
            roots.delete(id);
        },
        render: (children: React.ReactNode) => reconciler.updateContainer(children, root, null, null)
    }
}


export const createElement = (type: IcedTag, { children, ...props }: { children: IcedNode[] }): IcedNode | IcedText => {
    if (type == "text") {
        return {
            text: children[0] as never as string
        }
    }

    return {
        type,
        children,
        props,
    }
}