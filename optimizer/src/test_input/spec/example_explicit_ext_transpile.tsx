import { component$, $, useStyles$ } from '@qwik.dev/core';

export const App = component$((props) => {
	useStyles$('hola');
	return $(() => (
		<div></div>
	));
});
