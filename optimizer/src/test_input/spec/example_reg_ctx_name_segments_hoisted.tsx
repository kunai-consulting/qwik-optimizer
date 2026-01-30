import { $, component$, server$, useStyle$ } from '@qwik.dev/core';

export const Works = component$((props) => {
	useStyle$(STYLES);
	const text = 'hola';
	return (
		<div onClick$={server$(() => console.log('in server', text))}></div>
	);
});

const STYLES = '.class {}';
