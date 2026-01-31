import { component$, useSignal } from "@qwik.dev/core";

export const FieldInput = component$(() => {
  const localValue = useSignal(false);

  return (
    <input
      bind:checked={localValue}
      onInput$={() => {
        console.log("test");
      }}
    />
  );
});
