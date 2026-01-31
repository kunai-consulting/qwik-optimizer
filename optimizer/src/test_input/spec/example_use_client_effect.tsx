import { component$, useBrowserVisibleTask$, useStore, useStyles$ } from '@qwik.dev/core';

export const Child = component$(() => {
	const state = useStore({
		count: 0
	});

	// Double count watch
	useBrowserVisibleTask$(() => {
		const timer = setInterval(() => {
		state.count++;
		}, 1000);
		return () => {
		clearInterval(timer);
		}
	});

	return (
		<div>
		{state.count}
	</div>
	);
});
