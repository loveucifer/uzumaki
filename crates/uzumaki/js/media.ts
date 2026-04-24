import { Plugins } from './plugins';

export interface MediaLoadOptions {
  source: string;
  enableVideo?: boolean;
  enableAudio?: boolean;
}

export interface MediaPlayerOptions {
  backend?: string;
}

export interface MediaSnapshot {
  source: string | null;
  isLoaded: boolean;
  isPlaying: boolean;
  enableVideo: boolean;
  enableAudio: boolean;
  currentTimeMs: number;
  durationMs: number;
  volume: number;
  muted: boolean;
}

export interface CodecSupport {
  video: string[];
  audio: string[];
}

export interface VideoFramePacket {
  ptsMs: number;
  width: number;
  height: number;
  pixelFormat: string;
  bytes: Uint8Array;
}

export interface AudioPacket {
  ptsMs: number;
  sampleRate: number;
  channels: number;
  sampleFormat: string;
  bytes: Uint8Array;
}

interface NativeMediaPlayer {
  backend(): string;
  setBackend(backend: string): string;
  load(options: MediaLoadOptions): MediaSnapshot;
  play(): MediaSnapshot;
  pause(): MediaSnapshot;
  stop(): MediaSnapshot;
  seek(positionMs: number): MediaSnapshot;
  setVolume(volume: number): MediaSnapshot;
  setMuted(muted: boolean): MediaSnapshot;
  snapshot(): MediaSnapshot;
  tick(deltaMs: number): MediaSnapshot;
  readVideoFrame(): VideoFramePacket | null;
  readAudioPacket(): AudioPacket | null;
  codecSupport(): CodecSupport;
}

interface MediaNapiModule {
  MediaPlayer: new (options?: MediaPlayerOptions) => NativeMediaPlayer;
  getCodecSupport(): CodecSupport;
  supportsVideoCodec(codec: string): boolean;
  supportsAudioCodec(codec: string): boolean;
  availableMediaBackends(): string[];
}

let mediaNapiModule: MediaNapiModule | null = null;

export function configureMediaNapi(module: MediaNapiModule): void {
  mediaNapiModule = module;
}

function requireNapiModule(): MediaNapiModule {
  if (mediaNapiModule) {
    return mediaNapiModule;
  }

  throw new Error(
    'Media N-API module is not configured. Build/load uzumaki_media_napi and call configureMediaNapi(...) first.',
  );
}

export class MediaCodecs {
  static support(): CodecSupport {
    return requireNapiModule().getCodecSupport();
  }

  static supportsVideo(codec: string): boolean {
    return requireNapiModule().supportsVideoCodec(codec);
  }

  static supportsAudio(codec: string): boolean {
    return requireNapiModule().supportsAudioCodec(codec);
  }
}

export class MediaBackends {
  static available(): string[] {
    return requireNapiModule().availableMediaBackends();
  }
}

export class MediaPlayer {
  private native: NativeMediaPlayer;

  constructor(options?: MediaPlayerOptions) {
    Plugins.require('mediaPlayback');
    this.native = new (requireNapiModule().MediaPlayer)(options);
  }

  backend(): string {
    return this.native.backend();
  }

  setBackend(backend: string): string {
    return this.native.setBackend(backend);
  }

  load(options: MediaLoadOptions): MediaSnapshot {
    Plugins.require('mediaDecode');
    return this.native.load(options);
  }

  play(): MediaSnapshot {
    return this.native.play();
  }

  pause(): MediaSnapshot {
    return this.native.pause();
  }

  stop(): MediaSnapshot {
    return this.native.stop();
  }

  seek(positionMs: number): MediaSnapshot {
    return this.native.seek(positionMs);
  }

  setVolume(volume: number): MediaSnapshot {
    return this.native.setVolume(volume);
  }

  setMuted(muted: boolean): MediaSnapshot {
    return this.native.setMuted(muted);
  }

  snapshot(): MediaSnapshot {
    return this.native.snapshot();
  }

  tick(deltaMs: number): MediaSnapshot {
    return this.native.tick(deltaMs);
  }

  readVideoFrame(): VideoFramePacket | null {
    return this.native.readVideoFrame();
  }

  readAudioPacket(): AudioPacket | null {
    return this.native.readAudioPacket();
  }

  codecSupport(): CodecSupport {
    return this.native.codecSupport();
  }
}
