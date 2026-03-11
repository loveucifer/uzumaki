import { Window } from 'uzumaki';
import { render } from 'uzumaki/react';
import { App } from './app';

const window = new Window('main', { title: 'uzumaki' });

render(window, <App />);
