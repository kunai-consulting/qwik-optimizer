import { $, component$, useTask$ } from '@qwik.dev/core';
import { CONST } from 'const';
export const Works = component$((props) => {
	const {countNested} = useStore({value:{count:0}}).value;
	const countNested2 = countNested;
	const {hello} = countNested2;
	const bye = hello.bye;
	const {ciao} = bye.italian;


	return (
		<div ciao={ciao} >{foo}</div>
	);
});
