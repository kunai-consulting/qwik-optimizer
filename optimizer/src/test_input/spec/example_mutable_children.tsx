import { component$, useStore, Slot, Fragment } from '@qwik.dev/core';
import Image from './image.jpg?jsx';

export function Fn1(props: Stuff) {
	return (
		<>
			<div>{prop < 2 ? <p>1</p> : <Stuff>2</Stuff>}</div>
		</>
	);
}

export function Fn2(props: Stuff) {
	return (
		<div>{prop.value && <Stuff></Stuff>}<div></div></div>
	);
}

export function Fn3(props: Stuff) {
	if (prop.value) {
		return (
			<Stuff></Stuff>
		);
	}
	return (
		<div></div>
	);
}

export function Fn4(props: Stuff) {
	if (prop.value) {
		return (
			<div></div>
		);
	}
	return (
		<Stuff></Stuff>
	);
}

export const Arrow = (props: Stuff) => <div>{prop < 2 ? <p>1</p> : <Stuff>2</Stuff>}</div>;

export const AppDynamic1 = component$((props: Stuff) => {
	return (
		<>
			<div>{prop < 2 ? <p>1</p> : <Stuff>2</Stuff>}</div>
		</>
	);
});
export const AppDynamic2 = component$((props: Stuff) => {
	return (
		<div>{prop.value && <Stuff></Stuff>}<div></div></div>
	);
});

export const AppDynamic3 = component$((props: Stuff) => {
	if (prop.value) {
		return (
			<Stuff></Stuff>
		);
	}
	return (
		<div></div>
	);
});

export const AppDynamic4 = component$((props: Stuff) => {
	if (prop.value) {
		return (
			<div></div>
		);
	}
	return (
		<Stuff></Stuff>
	);
});

export const AppStatic = component$((props: Stuff) => {
	return (
		<>
			<div>Static {f ? 1 : 3}</div>
			<div>{prop < 2 ? <p>1</p> : <p>2</p>}</div>

			<div>{prop.value && <div></div>}</div>
			<div>{prop.value && <Fragment><Slot></Slot></Fragment>}</div>
			<div>{prop.value && <><div></div></>}</div>
			<div>{prop.value && <Image/>}</div>
			<div>Static {f ? 1 : 3}</div>
			<div>Static</div>
			<div>Static {props.value}</div>
			<div>Static {stuff()}</div>
			<div>Static {stuff()}</div>
		</>
	);
});
