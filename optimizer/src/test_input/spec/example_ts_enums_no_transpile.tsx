import { component$, useStore } from '@qwik.dev/core';

export enum Thing {
	A,
	B
}

export const App = component$(() => {
	console.log(Thing.A);
	return (
		<>
			<p class="stuff">Hello Qwik</p>
		</>
	);
});
