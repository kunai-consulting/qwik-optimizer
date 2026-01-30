import { component$ as Component, $ as onRender, useStore } from '@qwik.dev/core';

export const App = Component((props) => {
	const state = useStore({thing: 0});

	return onRender(() => (
		<div>{state.thing}</div>
	));
});
