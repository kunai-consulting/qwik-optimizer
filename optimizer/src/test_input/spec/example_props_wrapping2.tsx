import { $, component$, useSignal } from '@qwik.dev/core';
export const Works = component$((props: { fromProps: number }) => {
	let fromLocal = useSignal(0);
	return (
		<div
			computed={fromLocal + props.fromProps}
			local={fromLocal}
			props-wrap={props.fromProps}
			props-only={{props: props.fromProps}}
			props={{props: props.fromProps, local: fromLocal}}
				>
		</div>
	);
});
