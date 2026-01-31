import { component$ } from '@qwik.dev/core';
import { deps } from 'deps';

export const Foo = component$(({foo}) => {
	useMount$(() => {
		if (false) {
			deps();
		}
	});
	return (
		<div />
	);
})
