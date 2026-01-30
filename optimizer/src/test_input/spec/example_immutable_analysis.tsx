import { component$, useStore, $ } from '@qwik.dev/core';
import importedValue from 'v';
import styles from './styles.module.css';

export const App = component$((props) => {
	const {Model} = props;
	const state = useStore({count: 0});
	const remove = $((id: number) => {
		const d = state.data;
		d.splice(
			d.findIndex((d) => d.id === id),
			1
		)
		});
	return (
		<>
			<p class="stuff" onClick$={props.onClick$}>Hello Qwik</p>
			<Div
				class={styles.foo}
				document={window.document}
				onClick$={props.onClick$}
				onEvent$={() => console.log('stuff')}
				transparent$={() => {console.log('stuff')}}
				immutable1="stuff"
				immutable2={{
					foo: 'bar',
					baz: importedValue ? true : false,
				}}
				immutable3={2}
				immutable4$={(ev) => console.log(state.count)}
				immutable5={[1, 2, importedValue, null, {}]}
			>
				<p>Hello Qwik</p>
			</Div>
			[].map(() => (
				<Model
					class={state}
					remove$={remove}
					mutable1={{
						foo: 'bar',
						baz: state.count ? true : false,
					}}
					mutable2={(() => console.log(state.count))()}
					mutable3={[1, 2, state, null, {}]}
				/>
			));
		</>
	);
});
