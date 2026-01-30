import { $, component$ } from '@qwik.dev/core';

export const Foo = component$((props) => {
	return (
		<div>
			<Button {...props} />
			<ButtonArrow {...props} />
		</div>
	);
}, {
	tagName: "my-foo",
});

export function Button({text, color}) {
	return (
		<button onColor$={color} onClick$={()=>console.log(text, color)}>{text}</button>
	);
}

export const ButtonArrow = ({text, color}) => {
	return (
		<button onColor$={color} onClick$={()=>console.log(text, color)}>{text}</button>
	);
}
