import type { Window } from '..';
import type { JSX } from './jsx/runtime';

export class ReactRenderer {}

export function render(window: Window, tree: JSX.Element) {
  console.log(tree);
}
