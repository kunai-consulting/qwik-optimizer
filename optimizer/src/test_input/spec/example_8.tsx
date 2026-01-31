import { $, component$ } from '@qwik.dev/core';

export const Header = component$(() => {
	return $((hola) => {
		const hola = this;
		const {something, styff} = hola;
		const hello = hola.nothere.stuff[global];
		return (
			<Header/>
		);
	});
});
