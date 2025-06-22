#!/usr/bin/env node
//@ts-check

import * as esbuild from "esbuild";

const prod = process.env.MODE === "prod";

await esbuild.build({
    target: "es2023",
    format: "cjs",
    outfile: `dist/extension.js`,
    entryPoints: ["src/extension.ts"],
    external: ["vscode"],
    bundle: true,
    sourcesContent: false,
    minify: prod,
    sourcemap: !prod,
    platform: "neutral",
    mainFields: ["module", "main"],
});
