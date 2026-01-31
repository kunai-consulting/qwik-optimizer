import { component$, useSignal, Signal } from '@qwik.dev/core';
const Foo = component$(function() {
  const data = useSignal<Signal<any>[]>([]);
  return <div>
	{data.value.map(row => (
	  <div onClick$={() => console.log(row.value.id)} onMouseOver$={() => console.log('over' + row.value.id)}>
		<p onClick$={() => console.log('inner' + row.value.id)}>{item.value.id}</p>
	  </div>
	))}
  </div>;
})
