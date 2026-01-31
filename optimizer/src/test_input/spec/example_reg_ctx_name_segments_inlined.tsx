import { $, component$, server$ } from '@qwik.dev/core';
export const Works = component$((props) => {
	const text = 'hola';
	return (
		<div onClick$={server$(() => console.log('in server', text))}></div>
	);
});
