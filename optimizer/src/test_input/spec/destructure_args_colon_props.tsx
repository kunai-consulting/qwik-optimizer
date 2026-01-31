import { component$ } from "@qwik.dev/core";
export default component$((props) => {
	const { 'bind:value': bindValue } = props;
	return (
		<>
		{bindValue}
		</>
	);
});
