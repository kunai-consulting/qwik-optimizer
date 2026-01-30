import { component$, useSignal } from "@qwik.dev/core";
export default component$((props) => {
	const { test, ...rest } = props;
	const test = useSignal(rest['bind:value']);
	return (
		<>
		{test.value}
		</>
	);
});
