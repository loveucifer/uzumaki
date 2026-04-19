import { Window } from 'uzumaki-ui';
import { render } from 'uzumaki-ui/react';
import { App } from './app';

const window = new Window('main', {
  width: 1100,
  height: 700,
  title: 'uzumaki — playground',
});

render(window, <App />);
