import { component$, useStore, useSignal } from '@qwik.dev/core';
const Parent = component$(() => {
const cart = useStore<Cart>([]);
const results = useSignal(['foo', 'bar']);

return (
  <div>
    <button id="first" onClick$={() => (results.value = ['item1', 'item2'])}></button>

    {results.value.map((item, key) => (
      <button
        id={'second-' + key}
        onClick$={() => {
          cart.push(item);
        }}
      >
        {item}
      </button>
    ))}
    <ul>
      {cart.map((item) => (
        <li>
          <span>{item}</span>
        </li>
      ))}
    </ul>
  </div>
);
});
