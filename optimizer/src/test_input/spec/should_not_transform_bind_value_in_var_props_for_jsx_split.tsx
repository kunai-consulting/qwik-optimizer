import { $, component$, useSignal } from "@qwik.dev/core";

export const FieldInput = component$((props) => {
	const input = useSignal("");

  return (
		<>
			{/* var props */}
			<input
				bind:value={input}
				{...props}
			/>
			{/* const props */}
			<input
				{...props}
				bind:value={input}
			/>
		</>
  );
});
