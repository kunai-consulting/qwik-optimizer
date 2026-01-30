import { $, component$, useStore } from '@qwik.dev/core';
export const useCounter = () => {
	return useStore({count: 0});
}

export const STEP = 1;

export const App = component$((props) => {
	const state = useCounter();
	const thing = useStore({thing: 0});
	const STEP_2 = 2;

	const count2 = state.count * 2;
	return (
		<div onClick$={() => state.count+=count2 }>
			<span>{state.count}</span>
			{buttons.map(btn => (
				<button
					onClick$={() => state.count += btn.offset + thing + STEP + STEP_2 + props.step}
				>
					{btn.name}
				</button>
			))}

		</div>

	);
})
