import { component$, $ } from '@qwik.dev/core';

export const Greeter = component$(() => {
	const value = useSignal(0);
	const checked = useSignal(false);
	const stuff = useSignal();
	return (
		<>
			<input bind:value={value} />
			<input bind:checked={checked} />
			<input bind:stuff={stuff} />
			<div>{value}</div>
			<div>{value.value}</div>
		</>

	)
});
