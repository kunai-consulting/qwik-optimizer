import { $, component$ } from '@qwik.dev/core';

export const [a, {b, v1: [c], d=v2, ...e}, f=v3, ...g] = obj;

const exp1 = 1;
const internal = 2;
export {exp1, internal as expr2};

export function foo() { }
export class bar {}

export default function DefaultFn() {}

export const Header = component$(() => {
	return $(() => (
		<Footer>
			<div>{a}{b}{c}{d}{e}{f}{exp1}{internal}{foo}{bar}{DefaultFn}</div>
			<div>{v1}{v2}{v3}{obj}</div>
		</Footer>
	))
});

export const Footer = component$();
