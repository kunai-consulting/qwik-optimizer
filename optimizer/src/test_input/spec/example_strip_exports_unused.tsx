import { component$ } from '@qwik.dev/core';
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
	return <div>cmp</div>
});
