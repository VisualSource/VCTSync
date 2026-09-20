// Serves the `react` specifier inside QuickJS.
//
// This is a façade, not a bundle: it forwards the single React instance that
// `iced-dom` owns (see the note there for why that module has to own it), so
// user scripts, the JSX runtime and the reconciler all share one copy. A
// second copy would give components a different hook dispatcher than the
// reconciler installs, and every hook would fail with "Invalid hook call".
//
// Built with `iced-dom` external, so the specifier survives into the output
// and the QuickJS loader resolves it to the already-evaluated module.
//
// The re-exports are written out by hand rather than as `export * from`
// because React ships as CommonJS: esbuild cannot see through a star
// re-export of a CJS module and silently emits a bundle exporting nothing.
import { React } from "iced-dom";

export default React;

export const {
    Activity,
    Children,
    Component,
    Fragment,
    Profiler,
    PureComponent,
    StrictMode,
    Suspense,
    ViewTransition,
    act,
    addTransitionType,
    cache,
    cacheSignal,
    captureOwnerStack,
    cloneElement,
    createContext,
    createElement,
    createRef,
    forwardRef,
    isValidElement,
    lazy,
    memo,
    startTransition,
    use,
    useActionState,
    useCallback,
    useContext,
    useDebugValue,
    useDeferredValue,
    useEffect,
    useEffectEvent,
    useId,
    useImperativeHandle,
    useInsertionEffect,
    useLayoutEffect,
    useMemo,
    useOptimistic,
    useReducer,
    useRef,
    useState,
    useSyncExternalStore,
    useTransition,
    version,
} = React;
