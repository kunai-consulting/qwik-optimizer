import { component$, useStore } from '@qwik.dev/core';

export const App = component$(() => {
	const store = useStore({
		count: 0,
		stuff: 0,
		nested: {
			count: 0
		}
	});
	const signal = useSignal(0);
	return (
		<Cmp
			prop={'true' + 1 ? 'true' : ''}
			count={store.count}
			nested={store.nested.count}
			signal={signal}
			store={store.stuff + 12}
			value={signal.formData?.get('username')}
		>
		</Cmp>
	);
});

export const Cmp = component$((props) => {
	return (
		<>
			<p data-value={props.count}>{props.nested.count}</p>
			<p>Value {props.count}<span></span></p>
		</>
	);
});
