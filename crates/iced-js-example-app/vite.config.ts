import { defineConfig } from "vite";


export default defineConfig({
    input: [
        "./js/view.tsx"
    ],
    build: {
        rolldownOptions: {
            external: ["react", "react/jsx-runtime", "iced-dom"]
        }
    }
});