import { component$ } from '@qwik.dev/core';
import { Foo } from './foo';

export default component$((props) => {
	return (
		<Foo s={Math.random()} {...props} hello {...globalThis.nothing} />
	);
});
