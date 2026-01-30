import { $, component$ } from '@qwik.dev/core';

export const Foo = component$(({foo}) => {
	const arg0 = 20;
	return $(() => {
		const fn = ({aaa}) => aaa;
		return (
			<div>
				{foo}{fn()}{arg0}
			</div>
		)
	});
})

export const Bar = component$(({bar}) => {
	return $(() => {
		return (
			<div>
				{bar}
			</div>
		)
	});
})
