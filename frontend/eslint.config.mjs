import { defineConfig } from "eslint/config";
import nextCoreWebVitals from "eslint-config-next/core-web-vitals";
import typescriptEslint from "@typescript-eslint/eslint-plugin";
import simpleImportSort from "eslint-plugin-simple-import-sort";
import tsParser from "@typescript-eslint/parser";
import path from "node:path";
import { fileURLToPath } from "node:url";
import js from "@eslint/js";
import { FlatCompat } from "@eslint/eslintrc";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const compat = new FlatCompat({
    baseDirectory: __dirname,
    recommendedConfig: js.configs.recommended,
    allConfig: js.configs.all
});

export default defineConfig([{
    extends: [
        ...nextCoreWebVitals,
        ...compat.extends("plugin:@typescript-eslint/recommended")
    ],

    plugins: {
        "@typescript-eslint": typescriptEslint,
        "simple-import-sort": simpleImportSort,
    },

    languageOptions: {
        parser: tsParser,
    },

    rules: {
        "react/no-unescaped-entities": "off",
        "simple-import-sort/imports": "warn",
        "@next/next/no-img-element": "off",
        "react/display-name": "off",
        "react-hooks/exhaustive-deps": "off",
        "space-infix-ops": "warn",
        "comma-spacing": "warn",
        "react-hooks/rules-of-hooks": "off",
        eqeqeq: "error",
        "no-return-await": "warn",
        "no-var": "error",
        "prefer-const": "warn",
        "eol-last": ["warn", "always"],
        indent: ["warn", 2],
        semi: ["error", "never"],
        "arrow-parens": ["error", "as-needed"],
        "jsx-quotes": ["warn", "prefer-double"],
    },
}, {
    files: ["**/*.ts", "**/*.tsx"],

    rules: {
        "simple-import-sort/imports": ["warn", {
            groups: [["^react", "^next", "^@", "^[a-z]"], ["^/src/"], ["^./", "^.", "^../"]],
        }],
    },
}]);