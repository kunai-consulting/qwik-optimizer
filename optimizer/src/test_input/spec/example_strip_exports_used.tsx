import { component$, useResource$ } from '@qwik.dev/core';
import mongodb from 'mongodb';

export const onGet = () => {
	const data = mongodb.collection.whatever;
	return {
		body: {
		data
		}
	}
};

export default component$(()=> {
	useResource$(() => {
		return onGet();
	})
	return <div>cmp</div>
});
