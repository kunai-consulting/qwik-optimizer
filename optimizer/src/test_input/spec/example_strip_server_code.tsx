import { component$, serverLoader$, serverStuff$, $, client$, useStore, useTask$ } from '@qwik.dev/core';
import { isServer } from '@qwik.dev/core';
import mongo from 'mongodb';
import redis from 'redis';
import { handler } from 'serverless';

export const Parent = component$(() => {
	const state = useStore({
		text: ''
	});

	// Double count watch
	useTask$(async () => {
		if (!isServer) return;
		state.text = await mongo.users();
		redis.set(state.text);
	});

	serverStuff$(async () => {
		// should be removed too
		const a = $(() => {
			// from $(), should not be removed
		});
		const b = client$(() => {
			// from clien$(), should not be removed
		});
		return [a,b];
	})

	serverLoader$(handler);

	useTask$(() => {
		// Code
	});

	return (
		<div onClick$={() => console.log('parent')}>
			{state.text}
		</div>
	);
});
