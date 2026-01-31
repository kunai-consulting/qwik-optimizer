import { component$, $ } from '@qwik.dev/core';
export const Cmp = component$(() =>
	<Cmp foo={$(() => console.log('hi there'))}>Hello Qwik</Cmp>);
