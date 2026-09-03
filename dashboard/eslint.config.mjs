import next from "eslint-config-next/core-web-vitals";

/**
 * Flat config, required by ESLint 9's default resolution. Replaces the old
 * .eslintrc.json, which the eslintrc format made unreadable to ESLint 9+.
 *
 * @type {import("eslint").Linter.Config[]}
 */
const config = [
  {
    // Build output and the generated ambient types are not ours to lint.
    ignores: [".next/**", "out/**", "next-env.d.ts"],
  },
  ...next,
];

export default config;
