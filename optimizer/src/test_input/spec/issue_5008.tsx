import { component$, useStore } from "@qwik.dev/core";

export default component$(() => {
const store = useStore([{ value: 0 }]);
return (
	<>
	<button onClick$={() => store[0].value++}>+1</button>
	{store.map(function (v, idx) {
		return <div key={"fn_" + idx}>Function: {v.value}</div>;
	})}
	{store.map((v, idx) => (
		<div key={"arrow_" + idx}>Arrow: {v.value}</div>
	))}
	</>
);
});
