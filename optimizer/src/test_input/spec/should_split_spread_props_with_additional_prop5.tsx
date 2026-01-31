import { component$ } from '@qwik.dev/core';

function Hola(props: any) {
	return <div {...props}></div>;
}

export default component$(() => {
return <Hola>
	<div>1</div>
	<div>2</div>
</Hola>;
});
