---
title: Window
description: Creating and configuring application windows.
---

:::caution
Uzumaki is in alpha. This API is unstable and may change between releases.
:::

## Creating a window

Every Uzumaki app starts by creating a `Window` and passing it to `render`:

```tsx
import { Window } from 'uzumaki-ui';
import { render } from 'uzumaki-ui/react';

const window = new Window('main', {
  width: 800,
  height: 600,
  title: 'My App',
});

function App() {
  return (
    <view w="full" h="full" bg="#0f0f0f">
      <text color="#e4e4e7">Hello</text>
    </view>
  );
}

render(window, <App />);
```

## Window options

| Option   | Type     | Description                                               |
| -------- | -------- | --------------------------------------------------------- |
| `width`  | `number` | Initial window width in pixels                            |
| `height` | `number` | Initial window height in pixels                           |
| `title`  | `string` | Window title bar text                                     |
| `vars`   | `object` | Runtime theme variables resolved from `$token` style refs |

The first argument to `new Window()` is a window identifier string (e.g. `'main'`).

## Theme vars

Use `vars` for values that should switch per theme at runtime. Component styles
can reference these using `$token` values (for example from `defineVars(...)`).

```tsx
import { Window, defineVars, getWindow } from 'uzumaki-ui';

const { vars: darkVars, theme: darkTheme } = defineVars({
  bgBase: '#0f0f0f',
  textPrimary: '#e4e4e7',
});

const { vars: lightVars } = defineVars({
  bgBase: '#fafafa',
  textPrimary: '#111111',
});

const window = new Window('main', {
  width: 800,
  height: 600,
  title: 'Theme vars',
  vars: darkVars,
});

// Later, switch theme vars without a React rerender.
getWindow('main').setVars(lightVars);
```

`getWindow(label)` returns an existing window by label.

`window.setVars(vars)` updates the window variable table used by Rust to resolve
`$token` style values on the next frame.
