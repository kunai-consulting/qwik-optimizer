import { $, component$, useStore } from '@qwik.dev/core';
const Header = component$(() => {
	const thing = useStore();
	const {foo, bar} = foo();

	return (
		<div>{thing}</div>
	);
});
