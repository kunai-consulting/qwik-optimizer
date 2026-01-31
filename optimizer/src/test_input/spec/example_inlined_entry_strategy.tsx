import { component$, useBrowserVisibleTask$, useStore, useStyles$ } from '@qwik.dev/core';
import { thing } from './sibling';
import mongodb from 'mongodb';

export const Child = component$(() => {

	useStyles$('somestring');
	const state = useStore({
		count: 0
	});

	// Double count watch
	useBrowserVisibleTask$(() => {
		state.count = thing.doStuff() + import("./sibling");
	});

	return (
		<div onClick$={() => console.log(mongodb)}>
		</div>
	);
});
