import { $, component$, useSignal } from '@qwik.dev/core';
export const Works = component$(({fromProps}) => {
	let fromLocal = useSignal(0);
	return (
		<div>
			{fromLocal}
			{fromProps}
			{fromLocal + fromProps}
			{{props: fromProps}}
			{{local: fromLocal}}
			{{props: fromProps, local: fromLocal}}
		</div>
	);
});
