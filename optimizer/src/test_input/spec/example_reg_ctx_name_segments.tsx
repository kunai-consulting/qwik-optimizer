import { $, component$, server$ } from '@qwik.dev/core';
import { foo } from './foo';
export const Works = component$((props) => {
	const text = 'hola';
	return (
		<>
		<div onClick$={server$(() => console.log('in server', text))}></div>
		<div onClick$={() => foo()}></div>
		</>
	);
});
