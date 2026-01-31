import { component$, useTask$, useStore, useStyles$ } from '@qwik.dev/core';
import mongo from 'mongodb';
import redis from 'redis';

export const Parent = component$(() => {
	const state = useStore({
		text: ''
	});

	// Double count watch
	useTask$(async () => {
		state.text = await mongo.users();
		redis.set(state.text);
	});

	return (
		<div onClick$={() => console.log('parent')}>
			{state.text}
		</div>
	);
});

export const Child = component$(() => {
	const state = useStore({
		text: ''
	});

	// Double count watch
	useTask$(async () => {
		state.text = await mongo.users();
	});

	return (
		<div onClick$={() => console.log('child')}>
			{state.text}
		</div>
	);
});
