import { component$, useStore, mutable } from '@qwik.dev/core';

import {dep} from './file';

export const App = component$(() => {
	const signal = useSignal(0);
	const store = useStore({});
	return (
		<>
			<div>First text</div>
			<div>First {`text`}</div>
			<div>First {1}</div>
			<div>First {true}</div>
			<div>First {`text${12}`}</div>
			<div>First {typeof `text${12}` === 'string' ? 12 : 43}</div>
			<div>First {signal}</div>
			<div>First {signal.value}</div>
			<div>First {12 + signal.value}</div>
			<div>First {store.address.city.name}</div>
			<div>First {store.address.city.name ? 'true' : 'false'}</div>
			<div>First {dep}</div>
			<div>First {dep.thing}</div>
			<div>First {dep.thing + 'stuff'}</div>
			<div>First {globalThing}</div>
			<div>First {globalThing.thing}</div>
			<div>First {globalThing.thing + 'stuff'}</div>
			<div>First {signal.value()}</div>
			<div>First {signal.value + unknown()}</div>
			<div>First {mutable(signal)}</div>
			<div>First {signal.value + dep}</div>
		</>
	);
});
