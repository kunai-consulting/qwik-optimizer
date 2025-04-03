import { component$, useStyles$ } from '@qwik.dev/core';
import css1 from './global.css';
import css2 from './style.css';
import css3 from './style.css';

export const App = component$(() => {
    useStyles$(`${css1}${css2}`);
    useStyles$(css3);
})