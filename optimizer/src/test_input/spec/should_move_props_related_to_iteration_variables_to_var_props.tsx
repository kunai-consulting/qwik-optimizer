import { component$, useSignal } from "@qwik.dev/core";
import { TestComponent } from "./testComponent";

export const Child = component$(() => {
  const propCounterWithNested = useSignal(() => {
  return {
      data: [
        { attributeInArray: { counter: globalThis.propsCounter++ } },
        { attributeInArray: { counter: globalThis.propsCounter } },
      ],
    };
  })
  return (
    <div>
        {propCounterWithNested.value.data.map((item, index) => {
          return (
            <TestComponent
              counter={item.attributeInArray.counter}
              logString="Nested read through array"
              key={index}
            />
          );
        })}
    </div>
  );
})
