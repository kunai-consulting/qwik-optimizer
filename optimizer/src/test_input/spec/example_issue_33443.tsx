import { component$, useSignal } from '@qwik.dev/core';

export const Issue3742 = component$(({description = '', other}: any) => {
	const counter = useSignal(0);
	return (
		<div
		title={(description && 'description' in other) ? `Hello ${counter.value}` : `Bye ${counter.value}`}
		>
		Issue3742
		<button onClick$={() => counter.value++}>
			Increment
		</button>
		</div>
	)
	});
