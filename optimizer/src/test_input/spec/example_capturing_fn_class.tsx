import { $, component$ } from '@qwik.dev/core';

export const App = component$(() => {
	function hola() {
		console.log('hola');
	}
	class Thing {}
	class Other {}

	return $(() => {
		hola();
		new Thing();
		return (
			<div></div>
		)
	});
})
