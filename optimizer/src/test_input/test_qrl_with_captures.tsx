import { $, useSignal } from '@qwik.dev/core';

// QRL with captured variables
export const Counter = () => {
    const count = useSignal(0);
    const name = 'test';

    const increment = $(() => {
        // Captures count and name from enclosing scope
        console.log(count.value, name);
    });

    return increment;
};
