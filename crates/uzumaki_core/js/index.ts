import { Application, createWindow, pollEvents, setRemBase } from './bindings';
import { requestQuit } from './bindings';
import { eventManager, EventType } from './events';

export * from './window';

export { render } from './react';
export { eventManager, EventType } from './events';
export type {
  UzumakiEvent,
  UzumakiMouseEvent,
  UzumakiKeyboardEvent,
  UzumakiInputEvent,
  UzumakiFocusEvent,
} from './events';

interface AppEvent {
  type: string;
  windowId?: number;
  nodeId?: any;
  key?: string;
  width?: number;
  height?: number;
}

export async function runApp({
  entryFilePath,
  title = 'uzumaki',
  hot = false,
}: {
  entryFilePath: string;
  title?: string;
  hot?: boolean;
}) {
  process.env.WGPU_POWER_PREF = 'high';

  const app = new Application();

  let exiting = false;
  function shutdown() {
    if (exiting) {
      process.exit(1); // second signal, force kill
    }
    exiting = true;
    requestQuit();
  }

  process.on('SIGINT', shutdown);
  process.on('SIGTERM', shutdown);

  try {
    await import(entryFilePath);
  } catch (e) {
    console.error('Error running entry point');
    console.error(e);
    process.exit(1);
  }

  while (true) {
    const running = app.pumpAppEvents();

    const events: AppEvent[] = pollEvents();
    for (const event of events) {
      switch (event.type) {
        case 'mouseDown':
          if (event.nodeId != null) {
            eventManager.onRawEvent(EventType.MouseDown, event.nodeId, event);
          }
          break;
        case 'mouseUp':
          if (event.nodeId != null) {
            eventManager.onRawEvent(EventType.MouseUp, event.nodeId, event);
          }
          break;
        case 'click':
          if (event.nodeId != null) {
            eventManager.onRawEvent(EventType.Click, event.nodeId, event);
          }
          break;
        case 'keyDown':
          eventManager.onRawEvent(EventType.KeyDown, null, event);
          break;
        case 'keyUp':
          eventManager.onRawEvent(EventType.KeyUp, null, event);
          break;
        case 'input':
          if (event.nodeId != null) {
            eventManager.onRawEvent(EventType.Input, event.nodeId, event);
          }
          break;
        case 'focus':
          if (event.nodeId != null) {
            eventManager.onRawEvent(EventType.Focus, event.nodeId, event);
          }
          break;
        case 'blur':
          if (event.nodeId != null) {
            eventManager.onRawEvent(EventType.Blur, event.nodeId, event);
          }
          break;
        case 'resize':
          break;
        case 'hotReload':
          // todo this doesnt work :p
          console.log('[uzumaki] Hot reload');
          break;
      }
    }

    if (!running) break;

    await new Promise((resolve) => setImmediate(resolve));
  }

  app.destroy();
  console.log('Bye');
}
