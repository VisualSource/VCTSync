import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import { readFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";

/** Externalizes svg imports (resolving tsconfig paths), re-adds the `with { type: "svg" }`
 *  attribute rolldown strips off, and copies the referenced files into the bundle. */
function svgAssets(srcDir: string): Plugin {
    return {
        name: "svg-assets",
        enforce: "pre",
        async resolveId(source, importer) {
            if (!source.endsWith(".svg")) return null;
            const resolved = await this.resolve(source, importer, { skipSelf: true });
            return resolved && { id: resolved.id, external: "absolute" };
        },
        generateBundle(_options, bundle) {
            const emitted = new Set<string>();
            for (const file of Object.values(bundle)) {
                if (file.type !== "chunk") continue;
                const chunkDir = dirname(file.fileName);
                file.code = file.code.replace(
                    /(\bfrom\s*)(["'])([^"']+\.svg)\2(?!\s*with)/g,
                    (_match, from: string, quote: string, spec: string) => {
                        const abs = resolve(srcDir, chunkDir, spec);
                        const rel = relative(srcDir, abs).replaceAll("\\", "/");
                        if (!emitted.has(rel)) {
                            emitted.add(rel);
                            this.emitFile({ type: "asset", fileName: rel, source: readFileSync(abs) });
                        }
                        const out = relative(chunkDir, rel).replaceAll("\\", "/");
                        return `${from}${quote}${out.startsWith(".") ? out : `./${out}`}${quote} with { type: "svg" }`;
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
    resolve: {
        tsconfigPaths: true
    },
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
            external: ["react", "react/jsx-runtime", "react/compiler-runtime", "react-iced-native"]
        }
    }

});