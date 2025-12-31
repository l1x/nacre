import pluginJs from '@eslint/js';
import securityPlugin from 'eslint-plugin-security';
import tsPlugin from 'typescript-eslint';
import globals from 'globals';

/** @type {import('eslint').Linter.Config[]} */
export default [
  // Security
  securityPlugin.configs.recommended,
  {
    files: ['**/*.ts'],
  },
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node
      }
    },
  },
  {
    rules: {
      'no-console': 'warn',
      'no-eval': 'error',
      'no-implied-eval': 'error',
    },
  },
  // TypeScript Eslint
  {
    rules: {
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-unused-vars': 'warn',
    },
  },
  pluginJs.configs.recommended,
  ...tsPlugin.configs.recommended,
];
