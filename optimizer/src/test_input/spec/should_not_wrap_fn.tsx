import { component$, useSignal } from "@qwik.dev/core";
import { A } from "./componentA";

export const Cmp = component$(() => {
	const currentStep = useSignal('STEP_1');
	const currentType = useSignal<'NEXT' | 'PREVIOUS'>('PREVIOUS');

	const getStep = (step: string, type: 'NEXT' | 'PREVIOUS') => {
		return step === 'STEP_1' ? 'STEP_2' : 'STEP_1';
	};

	return (
		<>
			<button onClick$={() => (currentType.value = 'NEXT')}>CLICK</button>
			<A href={getStep(currentStep.value, currentType.value)} />
		</>
	);
});
