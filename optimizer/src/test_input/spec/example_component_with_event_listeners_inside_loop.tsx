import { $, component$, useStore, useSignal } from '@qwik.dev/core';
export const App = component$(() => {
      const cart = useStore<string[]>([]);
      const results = useSignal(['foo']);
      function loopArrowFn(results: string[]) {
        return results.map((item) => (
          <span
            onClick$={() => {
              cart.push(item);
            }}
          >
            {item}
          </span>
        ));
      }
      function loopForI(results: string[]) {
        const items = [];
        for (let i = 0; i < results.length; i++) {
          items.push(
            <span
              onClick$={() => {
                cart.push(results[i]);
              }}
            >
              {results[i]}
            </span>
          );
        }
        return items;
      }
      function loopForOf(results: string[]) {
        const items = [];
        for (const item of results) {
          items.push(
            <span
              onClick$={() => {
                cart.push(item);
              }}
            >
              {item}
            </span>
          );
        }
        return items;
      }
      function loopForIn(results: string[]) {
        const items = [];
        for (const key in results) {
          items.push(
            <span
              onClick$={() => {
                cart.push(results[key]);
              }}
            >
              {results[key]}
            </span>
          );
        }
        return items;
      }
      function loopWhile(results: string[]) {
        const items = [];
        let i = 0;
        while (i < results.length) {
          items.push(
            <span
              onClick$={() => {
                cart.push(results[i]);
              }}
            >
              {results[i]}
            </span>
          );
          i++;
        }
        return items;
      }
      return (
        <div>
          {results.value.map((item) => (
            <button
              id="second"
              onClick$={() => {
                cart.push(item);
              }}
            >
              {item}
            </button>
          ))}
          {loopArrowFn(results.value)}
          {loopForI(results.value)}
          {loopForOf(results.value)}
          {loopForIn(results.value)}
          {loopWhile(results.value)}
        </div>
      );
    });
