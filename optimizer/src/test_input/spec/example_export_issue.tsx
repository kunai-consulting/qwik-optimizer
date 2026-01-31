import { component$ } from '@qwik.dev/core';

const App = component$(() => {
	return (
		<div>hola</div>
	);
});


export const Root = component$((props: Stuff) => {
	return (
		<App/>
	);
});

const Other = 12;
export { Other as App };

export default App;
