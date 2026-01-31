import { $, component$, h, Fragment } from '@qwik.dev/core';

export const Lightweight = (props) => {
	return (
		<div>
			<>
				<div/>
				<button {...props}/>
			</>
		</div>
	)
};

export const Foo = component$((props) => {
	return $(() => {
		return (
			<div>
				<>
					<div class="class"/>
					<div class="class"></div>
					<div class="class">12</div>
				</>
				<div class="class">
					<Lightweight {...props}/>
				</div>
				<div class="class">
					<div/>
					<div/>
					<div/>
				</div>
				<div class="class">
					{children}
				</div>
			</div>
		)
	});
}, {
	tagName: "my-foo",
});
