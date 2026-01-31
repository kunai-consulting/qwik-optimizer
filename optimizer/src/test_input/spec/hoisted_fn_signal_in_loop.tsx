import { component$ } from '@qwik.dev/core';

export const App = component$(() => {
  const data = { value: [
    { value: { id: 1, selected: { value: true } } },
    { value: { id: 2, selected: { value: false } } },
    { value: { id: 3, selected: { value: true } } }
  ]};
  
  return (
    <table>
      {data.value.map((row) => {
        return (
          <tr
            key={row.value.id}
            class={row.value.selected.value ? "danger" : ""}
          >
            <td>{row.value.id}</td>
          </tr>
        );
      })}
    </table>
  );
});
