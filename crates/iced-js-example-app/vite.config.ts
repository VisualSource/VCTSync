import { defineConfig } from "vite";

export default defineConfig({
    input: "./js/view.tsx",
    define: {
        'process.env.NODE_ENV': '"production"'
    },
    build: {
        lib: {
            name: "app",
            fileName: "app",
            formats: ["es"],
        },
        emptyOutDir: false,
        
        minify: true,
        rolldownOptions: {
            
            output: {
                // minify: true
            },
            external: ["react", "react/jsx-runtime", "iced-dom",/\.svg$/]
        }
    }
});