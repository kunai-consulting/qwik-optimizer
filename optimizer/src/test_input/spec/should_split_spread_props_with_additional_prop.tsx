import { component$ } from '@qwik.dev/core';

export default component$((props) => {
	return (
		<div {...props} test="test"></div>
	);
});
