import { component$ } from '@qwik.dev/core';

// Function declaration component$ - transforms correctly
export const Counter = component$(function Counter() {
    return (
        <div>
            Counter component
        </div>
    );
});
