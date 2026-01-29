import { component$, $, useSignal } from '@qwik.dev/core';

// Nested component with handler - tests parent segment linking
export const Counter = component$(() => {
    const count = useSignal(0);

    return (
        <div>
            <button onClick$={() => {
                count.value++;
            }}>
                {count.value}
            </button>
        </div>
    );
});
