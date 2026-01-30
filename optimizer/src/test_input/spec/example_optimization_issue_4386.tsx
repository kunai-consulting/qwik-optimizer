import { component$ } from '@qwik.dev/core';

export const FOO_MAPPING = {
	A: 1,
	B: 2,
	C: 3,
	};

	export default component$(() => {
	const key = 'A';
	const value = FOO_MAPPING[key];

	return <>{value}</>;
	});
