import { Application, createWindow, pollEvents } from './bindings';
import { dispatchEvent } from './react/reconciler';
import { AppEventKind } from './bindings';
import { requestQuit } from './bindings';

export interface WindowAttributes {
  width: number;
  height: number;
  title: string;
}

export class Window {
  private _width: number;
  private _height: number;
  private _label: string;

  constructor(
    label: string,
    {
      width = 800,
      height = 600,
      title = 'uzumaki',
    }: Partial<WindowAttributes> = {},
  ) {
    this._width = width;
    this._height = height;
    this._label = label;

    createWindow({ width, height, label, title });
  }

  close() {}

  setSize(width: number, height: number) {
    this._width = width;
    this._height = height;
  }

  get width(): number {
    return this._width;
  }

  get height(): number {
    return this._height;
  }

  get label(): string {
    return this._label;
  }
}

export { render } from './react';

export async function runApp({
  entryFilePath,
  title = 'uzumaki',
}: {
  entryFilePath: string;
  title?: string;
}) {
  const app = new Application();

  function shutdown() {
    requestQuit();
    setTimeout(() => {
      process.exit(0);
    }, 1000);
  }

  process.on('SIGINT', shutdown);
  process.on('SIGTERM', shutdown);

  // we should do it later ?
  try {
    await import(entryFilePath);
  } catch (e) {
    console.error('Error running entry point');
    console.error(e);
    process.exit(1);
  }

  // Main loop: pump winit events, then drain and dispatch JS events.
  while (true) {
    const running = app.pumpAppEvents();
    if (!running) break;

    const events = pollEvents();
    for (const event of events) {
      if (event.kind === AppEventKind.DomEvent && event.domEvent) {
        dispatchEvent(event.domEvent.nodeId, event.domEvent.eventType);
      }
    }

    await new Promise((resolve) => setImmediate(resolve));
  }

  console.log('Bye');
}
