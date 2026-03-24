import * as core from './bindings';
import { eventManager, UzumakiEvent } from './events';

const windowRegistry = new Map<string, Window>();

type EventHandler = (ev: UzumakiEvent) => void;

export interface WindowAttributes {
  width: number;
  height: number;
  title: string;
}

export class Window {
  private _width!: number;
  private _height!: number;
  private _label!: string;
  private _id!: number;
  private _remBase: number = 16;
  private _eventId!: string;

  constructor(
    label: string,
    {
      width = 800,
      height = 600,
      title = 'uzumaki',
    }: Partial<WindowAttributes> = {},
  ) {
    // Return existing window: for hot reload
    const existing = windowRegistry.get(label);
    if (existing) {
      return existing;
    }

    this._width = width;
    this._height = height;
    this._label = label;
    this._id = core.createWindow({ width, height, title });
    this._eventId = `__window_${this._id}`;
    windowRegistry.set(label, this);
  }

  close() {
    eventManager.clearNode(this._eventId);
  }

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

  get id(): number {
    return this._id;
  }

  get eventId(): string {
    return this._eventId;
  }

  get remBase(): number {
    return this._remBase;
  }

  set remBase(value: number) {
    this._remBase = value;
    core.setRemBase(this._id, value);
  }

  on(eventName: string, handler: EventHandler): void {
    eventManager.addHandlerByName(this._eventId, eventName, handler);
  }

  off(eventName: string, handler: EventHandler): void {
    eventManager.removeHandlerByName(this._eventId, eventName, handler);
  }
}
