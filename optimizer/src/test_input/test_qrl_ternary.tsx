import { $ } from '@qwik.dev/core';

// QRL in ternary expression
const condition = true;

export const handler = condition
    ? $(() => console.log('true branch'))
    : $(() => console.log('false branch'));
