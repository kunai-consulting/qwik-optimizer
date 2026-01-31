import { component$, $ } from '@qwik.dev/core';
import { CustomComponent } from './custom-component';
import { AnotherComponent } from './another-component';

export const Greeter = component$(() => {
	return (
		<div>
			<CustomComponent onClick$={() => {}}/>
			{array.map(item => (
				<AnotherComponent onClick$={() => {}}/>
			))}
		</div>
	)
});
