import { $, component$, useStyles } from '@qwik.dev/core';
import { qrl } from '@qwik.dev/core/what';

export const hW = 12;
export const handleWatch = 42;

const componentQrl = () => console.log('not this', qrl());

componentQrl();
export const Foo = component$(() => {
	useStyles$('thing');
	const qwik = hW + handleWatch;
	console.log(qwik);
	const qrl = 23;
	return (
		<div onClick$={()=> console.log(qrl)}/>
	)
}, {
	tagName: "my-foo",
});

export const Root = component$(() => {
	useStyles($('thing'));
	return $(() => {
		return (
			<div/>
		)
	});
}, {
	tagName: "my-foo",
});
