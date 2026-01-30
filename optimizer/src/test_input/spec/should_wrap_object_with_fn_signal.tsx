import { component$ } from '@qwik.dev/core';
export default component$((props) => {
	// not destructure it so it is a var prop
	const item = props.something.count;
	return (
		<>
			<div data-no-wrap={item ? item * 2 : null} data-wrap={props.myobj.id + "test"}></div>
		</>
	);
});
