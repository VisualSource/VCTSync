// Serves the `react/jsx-runtime` specifier inside QuickJS.
//
// `tsconfig.json` sets `"jsx": "react-jsx"`, so compiled user scripts emit
// `import { jsx as _jsx } from "react/jsx-runtime"`, and that specifier has to
// resolve. Like `vendor/react.ts` this is a façade over the copy `react-iced-native`
// owns rather than a bundle of its own.
import { jsxRuntime } from "react-iced-native";

export const { jsx, jsxs, Fragment } = jsxRuntime;
