import { component$, $ } from '@qwik.dev/core';
import { hola } from 'sdfds';

export const Greeter = component$(() => {
	const stuff = useStore();
	return $(() => {
		return (
			<div
				class={{
					'foo': true,
					'bar': stuff.condition,
					'baz': hola ? 'true' : 'false',
				}}
			/>
		)
	});
});

const d = $(()=>console.log('thing'));
