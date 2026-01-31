import { component$, useComputed$ } from '@qwik.dev/core';
import { inlineTranslate } from 'translate-lib';

export default component$(() => {
	const t = inlineTranslate();

	const productTitle = useComputed$(() => {
		return 'Test title';
	});

	return (
		<img 
			attr={t('home.imageAlt.founded-product:')}
			alt={`${t('home.imageAlt.founded-product:')} ${productTitle.value}`} />
	);
});
