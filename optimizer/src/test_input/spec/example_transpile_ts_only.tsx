import { component$, useStore } from '@qwik.dev/core';

export const App = component$((props: Stuff) => {
	return (
		<Cmp>
			<p class="stuff" onClick$={() => console.log('warn')}>Hello Qwik</p>
		</Cmp>
	);
});
