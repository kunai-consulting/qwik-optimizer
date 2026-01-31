import { $, component$, useSignal } from "@qwik.dev/core";
import { destructureBindings } from "./destructure-bindings";

export const FieldInput = component$(
  (props) => {
    const initialValues = { value: undefined as string | undefined };
    const rest = destructureBindings(props, initialValues);

    return (
      <input
        {...rest}
        bind:value={finalValue}
		onClick$={() => {
			console.log('clicked');
		}}
      />
    );
  }
);
