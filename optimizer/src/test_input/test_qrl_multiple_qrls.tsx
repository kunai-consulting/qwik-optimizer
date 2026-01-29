import { $ } from '@qwik.dev/core';

// Multiple QRLs in same file - all get unique symbol names
export const handler1 = $(() => {
    console.log('handler1');
});

export const handler2 = $(() => {
    console.log('handler2');
});

export const handler3 = $(() => {
    console.log('handler3');
});
