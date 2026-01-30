import { component$, useSignal } from '@qwik.dev/core';


export default component$(() => {
const toggle = useSignal(true);
const t = (key: string) => key;
return (
	<button
		type="button"
		title={
		toggle.value !== ''
			? t('app.message.exists@@there is a message for you')
			: t('app.message.not_exists@@click to get a message!')
		}
	></button>
);
});
