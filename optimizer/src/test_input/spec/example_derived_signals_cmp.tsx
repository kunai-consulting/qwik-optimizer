import { component$, useStore, mutable } from '@qwik.dev/core';

import {dep} from './file';
import {Cmp} from './cmp';

export const App = component$(() => {
	const signal = useSignal(0);
	const store = useStore({});
	return (
		<Cmp
			staticText="text"
			staticText2={`text`}
			staticNumber={1}
			staticBoolean={true}
			staticExpr={`text${12}`}
			staticExpr2={typeof `text${12}` === 'string' ? 12 : 43}

			signal={signal}
			signalValue={signal.value}
			signalComputedValue={12 + signal.value}

			store={store.address.city.name}
			storeComputed={store.address.city.name ? 'true' : 'false'}

			dep={dep}
			depAccess={dep.thing}
			depComputed={dep.thing + 'stuff'}

			global={globalThing}
			globalAccess={globalThing.thing}
			globalComputed={globalThing.thing + 'stuff'}


			noInline={signal.value()}
			noInline2={signal.value + unknown()}
			noInline3={mutable(signal)}
			noInline4={signal.value + dep}
		/>
	);
});
