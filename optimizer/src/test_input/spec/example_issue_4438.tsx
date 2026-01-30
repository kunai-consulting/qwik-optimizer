import { component$, useSignal } from '@qwik.dev/core';

export const App = component$(() => {
	const toggle = useSignal(false);
	return (
		<>
			<div data-nu={toggle.value ? $localize`singular` : 'plural'}></div>
			<div>{toggle.value ? $localize`singular` : $localize`plural`}</div>
		</>
	);
});
