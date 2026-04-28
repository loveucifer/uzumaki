export type ThemeRefs<T extends Record<string, unknown>> = {
  [K in keyof T]: `$${Extract<K, string>}`;
};

export function defineVars<const T extends Record<string, unknown>>(
  tokens: T,
): {
  vars: T;
  theme: ThemeRefs<T>;
} {
  const theme = Object.fromEntries(
    Object.keys(tokens).map((k) => [k, `$${k}`]),
  ) as ThemeRefs<T>;

  return { vars: tokens, theme };
}
