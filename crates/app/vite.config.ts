import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

/** Re-adds the `with { type: "svg" }` attribute rolldown strips off external svg
 *  imports, and copies the referenced files into the bundle. */
function svgAssets(srcDir: string): Plugin {
    return {
        name: "svg-assets",
        generateBundle(_options, bundle) {
            const emitted = new Set<string>();
            for (const file of Object.values(bundle)) {
                if (file.type !== "chunk") continue;
                file.code = file.code.replace(
                    /(\bfrom\s*)(["'])(\.{1,2}\/[^"']+\.svg)\2(?!\s*with)/g,
                    (_match, from: string, quote: string, spec: string) => {
                        const rel = join(dirname(file.fileName), spec).replaceAll("\\", "/");
                        if (!emitted.has(rel)) {
                            emitted.add(rel);
                            this.emitFile({ type: "asset", fileName: rel, source: readFileSync(resolve(srcDir, rel)) });
                        }
                        return `${from}${quote}${spec}${quote} with { type: "svg" }`;
                    }
                );
            }
        }
    };
}

export default defineConfig({
    input: ["./view/app.tsx"],
    define: {
        'process.env.NODE_ENV': '"production"'
    },
    plugins: [
        react({
            compiler: true,
        }),
        svgAssets(resolve(import.meta.dirname, "view"))
    ],
    build: {
        lib: {
            name: "app",
            fileName: "app",
            formats: ["es"]
        },
        minify: true,
        rolldownOptions: {
            output: {
                minify: true,
            },
            external: ["react", "react/jsx-runtime", "react/compiler-runtime", "react-iced-native", /\.svg$/]
        }
    }

});