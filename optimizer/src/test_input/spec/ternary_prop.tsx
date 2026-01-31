import { component$, $, useSignal } from '@qwik.dev/core';
export const Cmp = component$(() => {
	const toggleSig = useSignal(false);

	const handleClick$ = $(() => {
		toggleSig.value = !toggleSig.value;
	});

	return (
		<button onClick$={handleClick$} data-open={toggleSig.value ? true : undefined}>
			Removing data-open re-renders
		</button>
	);
});
