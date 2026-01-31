import { component$ } from '@qwik.dev/core';

export const Issue3795 = component$(() => {
	let base = "foo";
	const firstAssignment = base;
	base += "bar";
	const secondAssignment = base;
	return (
		<div id='issue-3795-result'>{firstAssignment} {secondAssignment}</div>
	)
	});
