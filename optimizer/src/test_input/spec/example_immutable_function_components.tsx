import { component$, useStore, Slot } from '@qwik.dev/core';

export const App = component$((props: Stuff) => {
	return (
		<div>
			<Slot/>
		</div>
	);
});
