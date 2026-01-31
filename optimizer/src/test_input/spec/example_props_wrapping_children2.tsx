import { $, component$, useSignal } from '@qwik.dev/core';
export const Works = component$((props) => {
	let fromLocal = useSignal(0);
	return (
		<div>
		  before-
			{fromLocal}
			{props.fromProps}
			{fromLocal + props.fromProps}
			{{props: props.fromProps}}
			{{local: fromLocal}}
			{{props: props.fromProps, local: fromLocal}}
			-after
		</div>
	);
});
