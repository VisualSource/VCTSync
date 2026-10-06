// Serves the `react/compiler-runtime` specifier inside QuickJS.
//
// this is to helps resolve the commonjs to esm issues with react
import { compilerRuntime } from "react-iced-native";
//@ts-expect-error package does not export c from its types but its there
export const { c } = compilerRuntime;