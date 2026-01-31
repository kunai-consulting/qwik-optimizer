import { component$ } from '@qwik.dev/core';

export default component$((props: any) => {
	return <button {...props} onClick$={() => props.onClick$()}></button>;
});
