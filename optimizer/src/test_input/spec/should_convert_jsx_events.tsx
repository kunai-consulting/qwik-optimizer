import { component$ } from '@qwik.dev/core';

const ManyEventsComponent = component$(() => {
	return (
		<div>
			<button
				onClick$={() => {}}
				onDblClick$={() => {}}
			>
				click
			</button>
			<button
				onClick$={() => {}}
				onBlur$={() => {}}
				on-anotherCustom$={() => {}}
				document:onFocus$={() => {}}
				window:onClick$={() => {}}
			>
				click
			</button>
		</div>
	);
});
