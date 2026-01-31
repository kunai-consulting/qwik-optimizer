import { component$ } from '@qwik.dev/core';

export default component$(() => {
	return (
		<div onClick$={() => console.log(mongodb)}>
		</div>
	);
});
