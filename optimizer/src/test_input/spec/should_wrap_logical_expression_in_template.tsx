import { component$, useSignal } from '@qwik.dev/core';

export default component$(() => {
	const count = useSignal(0);
	const count2 = useSignal(0);
	return (
		<div>
			{(count || count2).value}
		</div>
	);
});
