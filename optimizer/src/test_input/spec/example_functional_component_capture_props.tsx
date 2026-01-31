import { $, component$, useStore } from '@qwik.dev/core';

export const App = component$(({count, rest: [I2, {I3, v1: [I4], I5=v2, ...I6}, I7=v3, ...I8]}) => {
	const state = useStore({count: 0});
	const {rest: [C2, {C3, v1: [C4], C5=v2, ...C6}, C7=v3, ...C8]} = foo();
	return $(() => {
		return (
			<div onClick$={() => state.count += count + total }>
				{I2}{I3}{I4}{I5}{I6}{I7}{I8}
				{C2}{C3}{C4}{C5}{C6}{C7}{C8}
				{v1}{v2}{v3}
			</div>
		)
	});
})
