import { component$, $ } from '@qwik.dev/core';
import { state } from './sibling';

export const Local = component$(() => {
	return (
		<div>{state}</div>
	)
});
