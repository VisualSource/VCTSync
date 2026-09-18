import { createContext } from "react";
import Reconciler, { type HostConfig, type ReactContext, type EventPriority } from "react-reconciler";


declare function setTimeout(fn: ()=>void,ms?: number): number;
declare function clearTimeout(id: number): void;
declare function queueMicrotask(cb: ()=>void) : void;

type IcedTag = string;

type IcedProps = Record<string,string>;
type IcedNode = { type: IcedTag, props: IcedProps, children: IcedChild[] };
type IcedText = { text: string }
type IcedChild = IcedNode | IcedText;
type IcedContainer = { commit(children: readonly IcedChild[]): void }


type IcedHostConfig = HostConfig<IcedTag,IcedProps,IcedContainer,IcedNode,IcedText,never,never,never,IcedNode,null,IcedChild[],number,-1,null>;

const DefaultEventPriority: EventPriority = 0;

let currentPriority: EventPriority = -1;
const config: IcedHostConfig = {
    supportsMutation: false,
    supportsHydration: false,
    supportsMicrotasks: true,
    supportsPersistence: true,
    

    /** Persistent Mode: main hooks  */
    createInstance(type, props, rootContainer, hostContext, internalHandle) {
        
    },
    createTextInstance(text, rootContainer, hostContext, internalHandle) {
        
    },
    appendInitialChild(parentInstance, child) {
        parentInstance.children.push(child);
    },
    cloneInstance(instance, type, oldProps, newProps, keepChildren, recyclableInstance) {
        
    },
    createContainerChildSet(container) {
        return [];
    },
    appendChildToContainer(container, child) {
        
    },
    finalizeContainerChildren(container, newChildren) {
        
    },
    replaceContainerChildren(container, newChildren) {
        
    },
    clearContainer(container) {
        
    },
    cloneHiddenInstance(instance, type, props, internalInstanceHandle) {
        
    },
    cloneHiddenTextInstance(instance, text, internalInstanceHandle) {
        
    },
    shouldSetTextContent(type, props) {
        return false;
    },
    getPublicInstance(instance) {
        
    },

    /** Context/commit hooks  */
    getRootHostContext(){
        return null;
    },
    getChildHostContext(parentHostContext, type, rootContainer) {
        return null;
    },  
    finalizeInitialChildren(instance, type, props, rootContainer, hostContext) {
        return false;
    },
    prepareForCommit(){
        return null;
    },
    resetAfterCommit(containerInfo) {
    
    },
    preparePortalMount(containerInfo) {
        
    },
    isPrimaryRenderer: true,


    /**Timers */
    scheduleTimeout: setTimeout,
    cancelTimeout: clearTimeout,
    noTimeout: -1,
    scheduleMicrotask:queueMicrotask,


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
        
    },
    afterActiveInstanceBlur() {
        
    },
    prepareScopeUpdate(scopeInstance, instance) {
        
    },
    getInstanceFromScope(scopeInstance) {
        return null;
    },
    detachDeletedInstance(node) {
        
    },
    resetFormInstance(form) {
        
    },
    requestPostPaintCallback(callback) {
        
    },
    shouldAttemptEagerTransition() {
        return false;    
    },
    trackSchedulerEvent() {
        
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
        
    },
    suspendInstance(type, props) {
        
    },
    waitForCommitToBeReady() {
        return null;
    },
    
};

const reconciler = Reconciler(config);

const onError = () => {}

const container: IcedContainer = {
    commit(children) {
        
    },
}

export const createRoot = () => {
    const root = reconciler.createContainer(container,1,null,false,null,"",onError,onError,onError,()=>{});

    return {
        render: (children: React.ReactNode) => reconciler.updateContainer(children,root,null,null)
    }
}
