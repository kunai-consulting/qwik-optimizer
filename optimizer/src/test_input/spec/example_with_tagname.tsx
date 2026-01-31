import { $, component$ } from '@qwik.dev/core';

export const Foo = component$(() => {
	return $(() => {
		return (
			<div>
			</div>
		)
	});
}, {
	tagName: "my-foo",
});
