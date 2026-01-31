import { $, component, onRender } from '@qwik.dev/core';

export const renderHeader = $(() => {
	return (
		<div onClick={$((ctx) => console.log(ctx))}/>
	);
});
const renderHeader = component($(() => {
	console.log("mount");
	return render;
}));
