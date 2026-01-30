import { component$, useStore, useTask$ } from '@qwik.dev/core';
import { useForm, useForm2 } from './some-file.ts';

export const Input = component$<{error: string, error2: string, error3: string}>(
	(props) => {
		useTask$(({ track }) => {
			track(() => props.error);
			track(() => props.error2);
			track(() => props.error3);
		});

		return (
			<>
			</>
		);
	}
);

export default component$(() => {
	const [store, math] = [useStore({errors: {}}), Math.random()];
	const [[store2]] = [[useStore({errors: {}})]];
	const { store3, math4 } = { store3: useStore({errors: {}}), math4: Math.random() };
	const math2 = [Math.random()];
	const { math3 } = { math3: Math.random() };
	const [store4] = useForm();
	const {store5} = useForm2();

	return (
		<div>
			<button onClick$={() => {
				store.errors.test = store.errors.test ? undefined : 'ERROR TEST';
			}}>click</button>
			<Input 
				error={store.errors.test}
				error2={store2.errors.test}
				error3={store3.errors.test}
				error4={store4.errors.test}
				error5={store5.errors.test}
				math={math}
				math2={math2}
				math3={math3}
				math4={math4}
			/>
		</div>
	);
});
