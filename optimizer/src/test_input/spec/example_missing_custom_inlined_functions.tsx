import { component$ as Component, $ as onRender, useStore, wrap, useEffect } from '@qwik.dev/core';


export const useMemo$ = (qrt) => {
	useEffect(qrt);
};

export const App = component$((props) => {
	const state = useStore({count: 0});
	useMemo$(() => {
		console.log(state.count);
	});
	return $(() => (
		<div>{state.count}</div>
	));
});
