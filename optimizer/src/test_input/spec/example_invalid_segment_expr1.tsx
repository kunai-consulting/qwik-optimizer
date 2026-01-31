import { $, component$, useStyles$ } from '@qwik.dev/core';
import css1 from './global.css';
import css2 from './style.css';

export const App = component$(() => {
	const style = `${css1}${css2}`;
	useStyles$(style);
	const render = () => {
		return (
			<div></div>
		)
	};
	return $(render);
})
