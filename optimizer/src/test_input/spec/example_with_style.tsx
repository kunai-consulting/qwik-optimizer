import { $, component$, useStyles$ } from '@qwik.dev/core';

export const Foo = component$(() => {
	useStyles$('.class {}');
	return (
		<div class="class"/>
	);
}, {
	tagName: "my-foo",
});
