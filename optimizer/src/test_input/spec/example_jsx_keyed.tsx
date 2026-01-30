import { component$, useStore } from '@qwik.dev/core';

export const App = component$((props: Stuff) => {
	return (
		<>
			<Cmp key="stuff"></Cmp>
			<Cmp></Cmp>
			<Cmp prop="23"></Cmp>
			<Cmp prop="23" key={props.stuff}></Cmp>
			<p key={props.stuff}>Hello Qwik</p>
		</>
	);
});
