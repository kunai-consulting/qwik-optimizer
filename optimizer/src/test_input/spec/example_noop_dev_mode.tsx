import { component$, useStore, serverStuff$, $ } from '@qwik.dev/core';

export const App = component$(() => {
	const stuff = useStore();
	serverStuff$(async () => {
		// should be removed but keep scope
		console.log(stuff.count)
	})
	serverStuff$(async () => {
		// should be removed
	})

	return (
		<Cmp>
			<p class="stuff"
				shouldRemove$={() => stuff.count}
				onClick$={() => console.log('warn')}
			>
				Hello Qwik
			</p>
		</Cmp>
	);
});
