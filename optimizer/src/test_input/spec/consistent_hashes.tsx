import { component$, $ } from '@qwik.dev/core';
import mongo from 'mongodb';

export const Greeter = component$(() => {
	// Double count watch
	useTask$(async () => {
		await mongo.users();
	});
	return (
		<div>
			<div onClick$={() => {}}/>
			<div onClick$={() => {}}/>
			<div onClick$={() => {}}/>
		</div>
	)
});
