import { component$ } from '@qwik.dev/core';

export const App = component$(() => {
	console.log(function*(lo: any, t: any) {
	console.log(yield (yield lo)(t.href).then((r) => r.json()));
	});

	return <p>Hello Qwik</p>;
});
