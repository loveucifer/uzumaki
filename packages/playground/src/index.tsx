import { Window } from 'usumaki';
import { render } from 'usumaki/react';
import { App } from './app';

const window = new Window('main', { title: 'Usumaki' });

render(window, <App />);
