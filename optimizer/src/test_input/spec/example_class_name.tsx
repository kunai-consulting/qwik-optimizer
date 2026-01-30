import { component$ } from '@qwik.dev/core';

export const App2 = component$(() => {
	const signal = useSignal();
	const computed = signal.value + 'foo';
	return (
		<>
			<div className="hola"></div>
			<div className={signal.value}></div>
			<div className={signal}></div>
			<div className={computed}></div>

			<Foo className="hola"></Foo>
			<Foo className={signal.value}></Foo>
			<Foo className={signal}></Foo>
			<Foo className={computed}></Foo>
		</>
	);
});
