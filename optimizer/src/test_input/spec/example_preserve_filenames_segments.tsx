import { component$, useStore } from '@qwik.dev/core';

export const App = component$((props: Stuff) => {
	foo();
	return (
		<Cmp>
			<p class="stuff" onClick$={() => console.log('warn')}>Hello Qwik</p>
		</Cmp>
	);
});

export const foo = () => console.log('foo');
