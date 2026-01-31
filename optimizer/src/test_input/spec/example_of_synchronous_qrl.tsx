import { sync$, component$ } from "@qwik.dev/core";

export default component$(() => {
return (
	<>
		<input onClick$={sync$(function(event, target) {
			// comment should be removed
			event.preventDefault();
		})}/>
		<input onClick$={sync$((event, target) => {
			event.preventDefault();
		})}/>
		<input onClick$={sync$((event, target) => event.preventDefault())}/>
	</>
);
});
