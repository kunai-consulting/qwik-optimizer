import { $, component$ } from '@qwik.dev/core';
export const Header = component$(() => {
	console.log("mount");
	return (
		<div onClick={$((ctx) => console.log(ctx))}/>
	);
});
