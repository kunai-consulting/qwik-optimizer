import { component$, useSignal } from "@qwik.dev/core";

export const FieldInput = component$(() => {
  const localValue = useSignal("");

  return (
    <input
      onInput$={() => {
        console.log("test");
      }}
      bind:value={localValue}
    />
  );
});
