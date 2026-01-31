import { component$ as Component, $ as onRender } from '@qwik.dev/core';

export const handler = $(()=>console.log('hola'));

export const App = component$((props) => {
	useStyles$('hola');
	return $(() => (
		<div>{state.thing}</div>
	));
});
