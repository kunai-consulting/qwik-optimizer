import { component$, useStore } from '@qwik.dev/core';

export default component$(() => {
	const panelStore = useStore(() => ({
		active: 'Input',
		list: PANELS,
	}));

	return (
		<div
			stuff={panelStore.active ? 'yes' : 'no'}
			class={{
				'too-long-to-wrap': true,
				'examples-panel-input': panelStore.active === 'Input',
				'examples-panel-output': panelStore.active === 'Output',
				'examples-panel-console': panelStore.active === 'Console',
			}}
		/>
	);
});
export const PANELS: ActivePanel[] = ['Examples', 'Input', 'Output', 'Console'];
