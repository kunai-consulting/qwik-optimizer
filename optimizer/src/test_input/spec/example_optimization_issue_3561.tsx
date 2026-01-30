import { component$ } from '@qwik.dev/core';

export const Issue3561 = component$(() => {
	const props = useStore({
		product: {
		currentVariant: {
			variantImage: 'image',
			variantNumber: 'number',
			setContents: 'contents',
		},
		},
	});
	const {
		currentVariant: { variantImage, variantNumber, setContents } = {},
	} = props.product;

	console.log(variantImage, variantNumber, setContents)

	return <p></p>;
	});
