import { component$, useSignal } from '@qwik.dev/core';

export default component$(() => {
	const count = useSignal(0);
	return (
		<div>
			{(count as any).value}
		</div>
	);
});
