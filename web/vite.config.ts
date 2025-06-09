import {defineConfig} from "vite";

export default defineConfig({
    optimizeDeps:{exclude:['@thenick775/mgba-wasm']},
    base: "",
    server:{
        headers:{
            "Cross-Origin-Embedder-Policy": "require-corp",
            "Cross-Origin-Opener-Policy": "same-origin"
        }
    },
});
