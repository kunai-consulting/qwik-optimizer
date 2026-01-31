import { $, component$ } from '@qwik.dev/core';
import thing from "../state";

export function foo() {
	return import("../foo/state2")
}

export const Header = component$(() => {
	return (
		<div>
			{import("../folder/state3")}
			{thing}
		</div>
	);
});
