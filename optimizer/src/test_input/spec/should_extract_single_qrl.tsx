import { $, component$, useSignal, Signal } from '@qwik.dev/core';
export const App = component$(() => {
	const data = useSignal<Signal<any>[]>([]);
	const selectedItem = useSignal<any | null>(null);
    return (
        <div>
          {data.value.map((row) => (
              <tr
                key={untrack(() => row.value.id)}
                class={row.value.selected.value ? "danger" : ""}
              >
                <td class="col-md-1">{row.value.id}</td>
                <td class="col-md-4">
                  <a
                    onClick$={() => {
                      if (selectedItem.value) {
                        selectedItem.value.selected.value = false;
                      }
                      selectedItem.value = row.value;
                      row.value.selected.value = true;
                    }}
                  >
                    {row.value.label.value}
                  </a>
                </td>
                <td class="col-md-1">
                  <a
                    onClick$={() => {
                      const dataValue = untrack(() => data.value);
                      data.value = dataValue.toSpliced(
                        dataValue.findIndex((d) => d.value.id === row.value.id),
                        1,
                      );
                    }}
                  >
                    <span aria-hidden="true">x</span>
                  </a>
                </td>
                <td class="col-md-6" />
              </tr>
          ))}
        </div>
      );
    });
