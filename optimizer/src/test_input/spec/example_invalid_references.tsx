import { $, component$ } from '@qwik.dev/core';

const I1 = 12;
const [I2, {I3, v1: [I4], I5=v2, ...I6}, I7=v3, ...I8] = obj;
function I9() {}
class I10 {}

export const App = component$(({count}) => {
	console.log(I1, I2, I3, I4, I5, I6, I7, I8, I9);
	console.log(itsok, v1, v2, v3, obj);
	return $(() => {
		return (
			<I10></I10>
		)
	});
})
