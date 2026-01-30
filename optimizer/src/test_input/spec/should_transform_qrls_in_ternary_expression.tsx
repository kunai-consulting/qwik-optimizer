import { $, component$, useSignal } from "@qwik.dev/core";

export const FieldInput = component$(() => {
	const enabled = useSignal(false);
	const input = useSignal("");

  return (
     <input
		id="input"
		onInput$={
		enabled.value
			? $((ev, el) => {
				input.value = el.value;
			})
			: undefined
		}
		onFocus$={() => {
			enabled.value = true;
		}}
	/>
  );
});
