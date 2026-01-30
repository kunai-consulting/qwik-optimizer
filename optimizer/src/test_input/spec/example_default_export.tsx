import { component$ } from '@qwik.dev/core';
import { sibling } from './sibling';

export default component$(() => {
	return (
		<div onClick$={() => console.log(mongodb, sibling)}>
		</div>
	);
});
