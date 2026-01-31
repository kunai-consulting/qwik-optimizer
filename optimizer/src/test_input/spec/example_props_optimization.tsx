import { $, component$, useTask$ } from '@qwik.dev/core';
import { CONST } from 'const';
export const Works = component$(({
	count,
	some = 1+2,
	hello = CONST,
	stuff: hey,
	stuffDefault: hey2 = 123,
	...rest}) => {
	console.log(hey, some);
	useTask$(({track}) => {
		track(() => count);
		console.log(count, rest, hey, some, hey2);
	});
	return (
		<div some={some} params={{ some }} class={count} {...rest} override>{count}</div>
	);
});

export const NoWorks2 = component$(({count, stuff: {hey}}) => {
	console.log(hey);
	useTask$(({track}) => {
		track(() => count);
		console.log(count);
	});
	return (
		<div class={count}>{count}</div>
	);
});

export const NoWorks3 = component$(({count, stuff = hola()}) => {
	console.log(stuff);
	useTask$(({track}) => {
		track(() => count);
		console.log(count);
	});
	return (
		<div class={count}>{count}</div>
	);
});
