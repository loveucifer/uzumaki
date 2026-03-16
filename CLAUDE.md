# Uzumaki

A native desktop UI framework for JavaScript/TypeScript. Uses **Bun** as the JS runtime, **NAPI** for Rust↔JS bindings, **wgpu** for GPU rendering, and **Vello** for 2D vector graphics. The React reconciler lets you write native desktop apps with JSX.

## Architecture

```
crates/uzumaki_core/     — Rust native core (NAPI addon)
  src/lib.rs             — Application, window management, DOM, event loop (winit)
  src/element.rs         — DOM tree, nodes, layout (taffy)
  src/gpu.rs             — wgpu GPU context
  src/window.rs          — Window surface + Vello rendering
  src/style.rs           — Style types (Length, Color, Edges, Corners, etc.)
  src/interactivity.rs   — Hit testing, hover/active states
  src/text.rs            — Text shaping/layout
  js/index.ts            — JS-side Window class + runApp() event loop
  js/react/              — Custom React reconciler
    reconciler.ts        — react-reconciler host config (UElement, mutations)
    jsx/runtime.ts       — JSX intrinsic element types + prop mapping

crates/refineable/       — Derive macro crate for style refinement

packages/playground/     — Example app (counter, dashboard)
```

## How it works

1. `runApp()` creates an `Application` (Rust), starts the winit event loop via `pump_app_events()`
2. JS creates windows via `new Window(label, opts)` → native `createWindow()`
3. React reconciler renders JSX → calls native DOM operations (`createElement`, `appendChild`, `setText`, `setProp`, etc.)
4. Props are sent to Rust through typed setters: `setLengthProp`, `setColorProp`, `setF32Prop`, `setEnumProp`
5. Rust runs taffy layout + Vello rendering each frame via wgpu

## JSX Syntax

Intrinsic elements: `<view>`, `<text>`, `<p>`, `<button>`

### Props

**Layout:** `w`, `h`, `p`, `px`, `py`, `pt`, `pb`, `pl`, `pr`, `m`, `mx`, `my`, `mt`, `mb`, `ml`, `mr`, `gap`

**Flexbox:** `flex`, `flexDir` (`"row"` | `"col"`), `flexGrow`, `flexShrink`, `items` (`"center"` | `"start"` | `"end"` | `"stretch"`), `justify` (`"center"` | `"between"` | `"around"` | `"evenly"`)

**Styling:** `bg`, `color`, `fontSize`, `fontWeight`, `rounded`, `roundedTL/TR/BR/BL`, `border`, `borderTop/Right/Bottom/Left`, `borderColor`, `opacity`, `display` (`"flex"` | `"none"` | `"block"`), `cursor`, `visible`

**State variants:** `hover:bg`, `hover:color`, `hover:opacity`, `hover:borderColor`, `active:bg`, `active:color`, `active:opacity`, `active:borderColor`

**Events:** `onClick`, `onMouseDown`, `onMouseUp`

### Value formats

- Numbers: treated as px (`w={100}`)
- Strings: `"100"` (px), `"2rem"`, `"50%"`, `"auto"`, `"full"` (= 100%)
- Colors: hex strings (`"#FF5733"`, `"#FF573380"`), `"transparent"`

### Example

```tsx
import { Window } from 'uzumaki';
import { render } from 'uzumaki/react';

const window = new Window('main', { width: 800, height: 600, title: 'My App' });

function App() {
  const [count, setCount] = useState(0);
  return (
    <view display="flex" flexDir="col" w="full" h="full" items="center" justify="center" bg="#0f0f0f">
      <text fontSize={24} color="#d4d4d4">Count: {count}</text>
      <view onClick={() => setCount(c => c + 1)} p="8" px="16" bg="#2d2d30" rounded="6" hover:bg="#373738">
        <text fontSize={16} color="#569cd6">Increment</text>
      </view>
    </view>
  );
}

render(window, <App />);
```

## Commands

```sh
pnpm start              # Build core + run playground
pnpm build:core         # Build the Rust NAPI addon
pnpm --filter playground start  # Run playground only
```

## Tech stack

- **Runtime:** Bun
- **Rust → JS:** napi-rs (N-API)
- **Windowing:** winit (pump_events mode for cooperative JS event loop)
- **Layout:** taffy (flexbox)
- **Rendering:** Vello + wgpu
- **React:** react-reconciler (mutation mode)
- **Build:** Cargo (Rust workspace) + pnpm (JS workspace)
