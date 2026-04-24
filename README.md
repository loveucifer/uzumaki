<div align="center">
  <img src="etc/logo.svg" width="140" alt="Uzumaki logo" />

  <h1>Uzumaki</h1>

  <p>Native desktop UI framework for JavaScript/TypeScript.<br/>
  GPU-rendered with <strong>wgpu</strong> + <strong>Vello</strong>, powered by a <strong>Deno</strong> runtime. Write your UI in React.</p>

[![CI](https://github.com/golok727/uzumaki/actions/workflows/ci.yml/badge.svg)](https://github.com/golok727/uzumaki/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE-APACHE)
[![npm](https://img.shields.io/npm/v/uzumaki-ui?color=cb3837&logo=npm)](https://www.npmjs.com/package/uzumaki-ui)
[![GitHub stars](https://img.shields.io/github/stars/golok727/uzumaki?style=flat&logo=github)](https://github.com/golok727/uzumaki/stargazers)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![Powered by Deno](https://img.shields.io/badge/runtime-Deno-black?logo=deno)](https://deno.com/)
[![wgpu](https://img.shields.io/badge/renderer-wgpu%20%2B%20vello-purple)](#)

</div>

> [!WARNING]
> Uzumaki is in alpha. The API is unstable and will change. Do not use in production.

## Quick Example

```tsx
import { useState } from 'react';
import { Window } from 'uzumaki-ui';
import { render } from 'uzumaki-ui/react';

const window = new Window('main', {
  width: 800,
  height: 600,
  title: 'My App',
});

function App() {
  const [count, setCount] = useState(0);

  return (
    <view
      display="flex"
      flexDir="col"
      w="full"
      h="full"
      items="center"
      justify="center"
      bg="#0f0f0f"
      gap={16}
    >
      <text fontSize={32} fontWeight={700} color="#e4e4e7">
        Welcome to Uzumaki
      </text>
      <text fontSize={18} color="#a1a1aa">
        Count: {count}
      </text>
      <view
        onClick={() => setCount((c) => c + 1)}
        p={10}
        px={24}
        bg="#2d2d30"
        rounded={8}
        hover:bg="#3e3e42"
        cursor="pointer"
      >
        <text fontSize={16} color="#60a5fa">
          Increment
        </text>
      </view>
    </view>
  );
}

render(window, <App />);
```

## Install

**macOS**

```sh
curl -fsSL https://uzumaki.run/install.sh | sh
```

**Windows**

```powershell
irm https://uzumaki.run/install.ps1 | iex
```

Then create a project:

```sh
uzumaki init my-app
cd my-app
bun install
bun dev
```

## Plugin Capabilities

Uzumaki can gate native extension features (for example bluetooth, camera, and
media decode) through `uzumaki.config.json`.

```json
{
  "plugins": {
    "allow": ["bluetooth", "mediaDecode", "mediaPlayback"],
    "deny": ["camera"]
  }
}
```

In app code, use capability checks before enabling feature paths:

```ts
import { Plugins } from 'uzumaki-ui';

if (Plugins.has('bluetooth')) {
  // Enable bluetooth feature flow.
}

// Throw a clear runtime error if a capability is required.
Plugins.require('mediaPlayback');
```

## Experimental N-API Media Module

The repository now includes a native N-API media addon in
`crates/media_napi` (crate name: `uzumaki_media_napi`).

Current status:

- Provides a `MediaPlayer` class API surface (`load`, `play`, `pause`, `stop`, `seek`, `setVolume`, `setMuted`, `snapshot`, `tick`)
- Exposes codec support helpers (`getCodecSupport`, `supportsVideoCodec`, `supportsAudioCodec`)
- Uses FFmpeg as the decoder backend when built with the `ffmpeg-backend` feature

From `uzumaki-ui`, wire the addon into the JS layer with `configureMediaNapi`:

```ts
import {
  MediaBackends,
  MediaPlayer,
  MediaCodecs,
  Plugins,
  configureMediaNapi,
} from 'uzumaki-ui';
import mediaNapi from './path-to-loaded-media-addon';

configureMediaNapi(mediaNapi);

Plugins.require('mediaPlayback');
Plugins.require('mediaDecode');

console.log(MediaBackends.available());
console.log(MediaCodecs.support());

const player = new MediaPlayer({ backend: 'ffmpeg' });
player.load({ source: './movie.mp4', enableAudio: true, enableVideo: true });
player.play();

// Framework pipeline contracts: decoded packets are exposed here.
player.tick(16);
const frame = player.readVideoFrame();
const audio = player.readAudioPacket();
```

The framework now has backend/pipeline contracts for decoder integration:

- Backend selection (`ffmpeg` when compiled with `ffmpeg-backend`)
- Unified frame packet pull API (`readVideoFrame`)
- Unified audio packet pull API (`readAudioPacket`)

To enable decoding, compile the media addon with the `ffmpeg-backend` feature.

## Links

- [Docs](https://uzumaki.run)
- [GitHub](https://github.com/golok727/uzumaki)
- [Contributing](CONTRIBUTING.md)
- [Development](DEVELOPMENT.md)
- [X / Twitter](https://x.com/golok727)

## License

Licensed under either [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option.
