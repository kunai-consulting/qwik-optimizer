import { component$, useSignal, Signal } from '@qwik.dev/core';
const Foo = component$(function() {
  const data = useSignal<Signal<any>[]>([]);
  const data2 = useSignal<Signal<any>[]>([]);
  return <div>
	{data.value.map(row => (
	  <div onClick$={() => console.log(row.value.id)}>
		{data2.value.map(item => (
		  <p onClick$={() => console.log(row.value.id, item.value.id)}>{row.value.id}-{item.value.id}</p>
		))}
	  </div>
	))}
  </div>;
})
