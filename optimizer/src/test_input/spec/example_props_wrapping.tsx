import { $, component$, useSignal } from '@qwik.dev/core';
export const Works = component$(({fromProps}) => {
	let fromLocal = useSignal(0);
	return (
		<div
			computed={fromLocal + fromProps}
			local={fromLocal}
			props-wrap={fromProps}
			props-only={{props: fromProps}}
			props={{props: fromProps, local: fromLocal}}
				>
		</div>
	);
});
