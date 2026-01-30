import { component$, $, useStore, wrap, useEffect } from '@qwik.dev/core';

export const useMemoQrl = (qrt) => {
	useEffect(qrt);
};

export const useMemo$ = wrap(useMemoQrl);

export const App = component$((props) => {
	const state = useStore({count: 0});
	useMemo$(() => {
		console.log(state.count);
	});
	return $(() => (
		<div>{state.count}</div>
	));
});

export const Lightweight = (props) => {
	useMemo$(() => {
		console.log(state.count);
	});
};
