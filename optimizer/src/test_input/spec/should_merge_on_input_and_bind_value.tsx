import { component$, useSignal } from "@qwik.dev/core";

export const FieldInput = component$(() => {
  const localValue = useSignal("");

  return (
    <input
      bind:value={localValue}
      onInput$={() => {
        console.log("test");
      }}
    />
  );
});
