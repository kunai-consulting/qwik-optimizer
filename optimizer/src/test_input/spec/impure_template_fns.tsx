import { component$, useSignal } from '@qwik.dev/core';
const useFoo = (count) => {
	const tag = (s) => {
		const value = typeof s === "string" ? s : s[0];
		return \`\${value}-\${count.value}\`;
	}
	return tag;
}

export default component$(() => {
	const count = useSignal(0);
	const foo = useFoo(count);
	return (
		<>
			<p>{foo("test")}</p>
			<p>{foo\`test\`}</p>
			<button onClick$={() => count.value++}>Count up</button>
		</>
	);
});
