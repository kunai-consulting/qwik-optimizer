import { component$ } from '@qwik.dev/core';

export const Foo = component$(() => {
	return (
		<div>
			<div onClick$={() => console.log('first')}/>
			<div onClick$={() => console.log('second')}/>
			<div onClick$={() => console.log('third')}/>
		</div>
	);
});
