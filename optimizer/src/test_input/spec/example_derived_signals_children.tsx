import { component$, useStore, mutable } from '@qwik.dev/core';

import {dep} from './file';

export const TextContent = component$((props) => {
	return (
		<>
			<div>data-nu: {props['data-nu']}</div>
			<div>class: {props.class}</div>
		</>
	);
});

export const App = component$(() => {
	const signal = useSignal(0);
	const store = useStore({});
	return (
		<>
			<div>text</div>
			<div>{`text`}</div>
			<div>{1}</div>
			<div>{true}</div>
			<div>{`text${12}`}</div>
			<div>{typeof `text${12}` === 'string' ? 12 : 43}</div>
			<div>{signal}</div>
			<div>{signal.value}</div>
			<div>{12 + signal.value}</div>
			<div>{store.address.city.name}</div>
			<div>{store.address.city.name ? 'true' : 'false'}</div>
			<div>{dep}</div>
			<div>{dep.thing}</div>
			<div>{dep.thing + 'stuff'}</div>
			<div>{globalThing}</div>
			<div>{globalThing.thing}</div>
			<div>{globalThing.thing + 'stuff'}</div>
			<div>{signal.value()}</div>
			<div>{signal.value + unknown()}</div>
			<div>{mutable(signal)}</div>
			<div>{signal.value + dep}</div>
		</>
	);
});
