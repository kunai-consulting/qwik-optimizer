import { $, component$, useSignal } from "@qwik.dev/core";

export const FieldInput = component$((props) => {
	const input = useSignal(true);

  return (
		<>
			{/* var props */}
			<input
				bind:checked={input}
				{...props}
			/>
			{/* const props */}
			<input
				{...props}
				bind:checked={input}
			/>
		</>
  );
});
