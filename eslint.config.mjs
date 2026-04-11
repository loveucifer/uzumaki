import js from '@eslint/js';
import prettierConfig from 'eslint-config-prettier';
import pluginPerfectionist from 'eslint-plugin-perfectionist';
import pluginPrettier from 'eslint-plugin-prettier';
import reactHooks from 'eslint-plugin-react-hooks';
import pluginUnicorn from 'eslint-plugin-unicorn';
import { defineConfig } from 'eslint/config';
import globals from 'globals';
import { readFileSync } from 'node:fs';
import tseslint from 'typescript-eslint';

const ignoreList = readFileSync(
  new URL('.prettierignore', import.meta.url),
  'utf8',
)
  .split(/\r?\n/u)
  .map((pattern) => pattern.trim())
  .filter(Boolean);

export default defineConfig(
  {
    ignores: [
      ...ignoreList,
      '**/*.local.*',
      '**/dist/**',
      'crates/uzumaki/js/generated/**',
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  prettierConfig,
  {
    files: ['**/*.{js,mjs,ts,tsx}'],
    languageOptions: {
      ecmaVersion: 'latest',
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    plugins: {
      '@typescript-eslint': tseslint.plugin,
      'react-hooks': reactHooks,
      perfectionist: pluginPerfectionist,
      prettier: pluginPrettier,
      unicorn: pluginUnicorn,
    },
    rules: {
      ...reactHooks.configs.flat.recommended.rules,
      ...pluginUnicorn.configs.recommended.rules,
      '@typescript-eslint/no-empty-object-type': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-unsafe-function-type': 'off',
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          args: 'all',
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
        },
      ],
      'no-unused-vars': 'off',
      'perfectionist/sort-imports': [
        'error',
        {
          order: 'asc',
          type: 'natural',
        },
      ],
      'prettier/prettier': 'error',
      'unicorn/filename-case': [
        'error',
        {
          cases: {
            camelCase: true,
            kebabCase: true,
          },
          ignore: [String.raw`^\d+_`],
        },
      ],
      'unicorn/no-abusive-eslint-disable': 'off',
      'unicorn/no-array-reduce': 'off',
      'unicorn/no-null': 'off',
      'unicorn/no-static-only-class': 'off',
      'unicorn/prevent-abbreviations': 'off',
    },
  },
  {
    files: ['crates/uzumaki/core/**/*.js'],
    languageOptions: {
      globals: {
        Deno: 'readonly',
      },
    },
  },
  {
    files: ['crates/uzumaki/js/react/jsx/types.ts'],
    rules: {
      '@typescript-eslint/no-namespace': 'off',
    },
  },
  {
    files: ['crates/uzumaki/js/react/useInput.ts'],
    rules: {
      'react-hooks/immutability': 'off',
      'react-hooks/refs': 'off',
    },
  },
  {
    files: [
      'scripts/**/*.ts',
      '**/*.config.{js,mjs,ts}',
      'crates/uzumaki/cli/**/*.ts',
    ],
    rules: {
      'unicorn/no-process-exit': 'off',
    },
  },
);
