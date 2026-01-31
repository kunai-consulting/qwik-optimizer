import { component$, useSignal } from "@qwik.dev/core";
export default component$((props) => {
	const { 'bind:value': bindValue } = props;
	const test = useSignal(bindValue);
	return (
		<>
		{test.value}
		</>
	);
});
