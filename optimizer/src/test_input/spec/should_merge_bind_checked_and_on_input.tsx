import { component$, useSignal } from "@qwik.dev/core";

export const FieldInput = component$(() => {
  const localValue = useSignal(false);

  return (
    <input
      onInput$={() => {
        console.log("test");
      }}
      bind:checked={localValue}
    />
  );
});
