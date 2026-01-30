import { component$, useStore, mutable } from '@qwik.dev/core';

import {dep} from './file';

export const App = component$(() => {
	const signal = useSignal(0);
	const store = useStore({});
	return (
		<>
			<ul id="issue-2800-result">
				{Object.entries(store).map(([key, value]) => (
				<li>
					{key} - {value}
				</li>
				))}
			</ul>
		</>
	);
});
