import { useRef, useEffect } from 'react';
import * as core from '../bindings';

export interface InputHandle {
  /** Read the current value from the Rust buffer. */
  readonly value: string;
  /** Write a new value to the Rust buffer + trigger repaint. */
  set(value: string): void;
  /** Focus this input programmatically. */
  focus(): void;
  /** @internal marker for reconciler */
  readonly __handle: true;
  /** @internal bound by reconciler on mount */
  __nodeId: any;
  /** @internal bound by reconciler on mount */
  __windowId: number | null;
  /** @internal onChange callback */
  __onChange: ((value: string) => void) | undefined;
}

export interface InputHandleOptions {
  onChange?: (value: string) => void;
}

/**
 * Create an InputHandle outside of React.
 * The handle is a stable pointer to a Rust allocation — it can live
 * in component state, external stores, context, or module scope.
 * The Rust buffer is always the source of truth.
 */
export function createInputHandle(
  initialValue?: string,
  options?: InputHandleOptions,
): InputHandle {
  const handle: InputHandle = {
    __handle: true as const,
    __nodeId: null,
    __windowId: null,
    __onChange: options?.onChange,

    get value(): string {
      if (handle.__windowId != null && handle.__nodeId != null) {
        return core.getInputValue(handle.__windowId, handle.__nodeId);
      }
      // Not yet mounted — return initial value or empty
      return initialValue ?? '';
    },

    set(value: string): void {
      if (handle.__windowId != null && handle.__nodeId != null) {
        core.setInputValue(handle.__windowId, handle.__nodeId, value);
        core.requestRedraw(handle.__windowId);
      }
    },

    focus(): void {
      if (handle.__windowId != null && handle.__nodeId != null) {
        core.focusInput(handle.__windowId, handle.__nodeId);
        core.requestRedraw(handle.__windowId);
      }
    },
  };

  // Store initial value for reconciler to apply on mount
  (handle as any).__initialValue = initialValue ?? '';

  return handle;
}

// Fixme claude cheated me this will no trigger a rerender properly ig take a look into it
/**
 * Component-local input handle. Creates a stable handle ref that
 * lives for the lifetime of the component. Same API as createInputHandle,
 * but tied to React's lifecycle.
 */
export function useInput(
  initialValue?: string,
  options?: InputHandleOptions,
): InputHandle {
  const ref = useRef<InputHandle | null>(null);
  if (ref.current === null) {
    ref.current = createInputHandle(initialValue, options);
  }
  // Keep onChange up to date without re-creating the handle
  useEffect(() => {
    if (ref.current) {
      ref.current.__onChange = options?.onChange;
    }
  });
  return ref.current;
}
