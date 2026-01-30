import { component$, useClientMount$, useStore, useTask$ } from '@qwik.dev/core';
import mongo from 'mongodb';
import redis from 'redis';
import threejs from 'threejs';
import { a } from './keep';
import { b } from '../keep2';
import { c } from '../../remove';

export const Parent = component$(() => {
	const state = useStore({
		text: ''
	});

	// Double count watch
	useClientMount$(async () => {
		state.text = await mongo.users();
		redis.set(state.text, a, b, c);
	});

	useTask$(() => {
		// Code
	});

	return (
		<div
			shouldRemove$={() => state.text}
			onClick$={() => console.log('parent', state, threejs)}
		>
			<Div
				onClick$={() => console.log('keep')}
				render$={() => state.text}
			/>
			{state.text}
		</div>
	);
});
