import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
    input: ["./view/app.tsx"],
    define: {
        'process.env.NODE_ENV': '"production"'
    },
    plugins: [
        react({
            compiler: true,
        })
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
            external: ["react", "react/jsx-runtime", "react-iced-native", /\.svg$/]
        }
    }

});