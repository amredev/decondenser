import tseslint from "typescript-eslint";
import { includeIgnoreFile } from "@eslint/compat";
import { fileURLToPath } from "node:url";

const gitignorePath = fileURLToPath(new URL(".gitignore", import.meta.url));

export default tseslint.config(
    includeIgnoreFile(gitignorePath),
    ...tseslint.configs.strictTypeChecked,
    ...tseslint.configs.stylisticTypeChecked,
    {
        languageOptions: {
            parserOptions: {
                projectService: {
                    allowDefaultProject: [
                        "eslint.config.mjs",
                        "decondenser-vscode/esbuild.mjs",
                    ],
                    defaultProject: "./tsconfig.base.json",
                },
                tsconfigRootDir: import.meta.dirname,
            },
        },
        rules: {
            "@typescript-eslint/no-explicit-any": "off",
            "@typescript-eslint/no-unused-vars": [
                "error",
                {
                    args: "all",
                    argsIgnorePattern: "^_",
                    caughtErrors: "all",
                    caughtErrorsIgnorePattern: "^_",
                    destructuredArrayIgnorePattern: "^_",
                    // Require `.+` after the underscore to prevent false-negatives
                    // for `lodash` imports, that are named as just `_`.
                    varsIgnorePattern: "^_.+",
                },
            ],
            "@typescript-eslint/consistent-type-definitions": [
                "error",
                "interface",
            ],
            "@typescript-eslint/no-empty-object-type": [
                "error",
                { allowInterfaces: "always" },
            ],

            // Rules with type info
            "@typescript-eslint/prefer-nullish-coalescing": "off",
            "@typescript-eslint/restrict-template-expressions": [
                "error",
                {
                    allowNumber: true,
                    allowNullish: false,
                },
            ],
            "@typescript-eslint/no-unnecessary-condition": [
                "error",
                {
                    allowConstantLoopConditions: "only-allowed-literals",
                    checkTypePredicates: true,
                },
            ],
        },
    },
);
