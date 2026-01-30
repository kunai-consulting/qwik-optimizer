import { $, component$, useSignal, Signal } from '@qwik.dev/core';
const Foo = component$(() => {
  const data = useSignal<Signal<any>[]>([]);
  const Inner = component$((props) => {
    const data = props.data
    return <div>{data.value.map(item => <p onClick$={() => console.log(item.value.id)}>{item.value.id}</p>)}</div>
  })
  return <Inner data={data} />
})
