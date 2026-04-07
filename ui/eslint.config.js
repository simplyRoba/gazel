import js from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import globals from "globals";
import svelte from "eslint-plugin-svelte";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: ["build/", ".svelte-kit/", "coverage/", "package-lock.json"],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...svelte.configs.recommended,
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    files: ["**/*.ts", "**/*.js", "**/*.svelte"],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: [".svelte"],
      },
    },
    rules: {
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    files: ["**/*.svelte"],
    languageOptions: {
      globals: {
        ...globals.browser,
      },
    },
    rules: {
      "no-undef": "off",
    },
  },
  eslintConfigPrettier,
);
