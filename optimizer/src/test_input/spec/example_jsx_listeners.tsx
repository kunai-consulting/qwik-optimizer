import { $, component$ } from '@qwik.dev/core';

export const Foo = component$(() => {

	return $(() => {
		const handler = $(() => console.log('reused'));
		return (
			<div
				onClick$={()=>console.log('onClick$')}
				onDocumentScroll$={()=>console.log('onDocumentScroll')}
				onDocumentScroll$={()=>console.log('onWindowScroll')}

				on-cLick$={()=>console.log('on-cLick$')}
				onDocument-sCroll$={()=>console.log('onDocument-sCroll')}
				onDocument-scroLL$={()=>console.log('onDocument-scroLL')}

				host:onClick$={()=>console.log('host:onClick$')}
				host:onDocumentScroll$={()=>console.log('host:onDocument:scroll')}
				host:onDocumentScroll$={()=>console.log('host:onWindow:scroll')}

				onKeyup$={handler}
				onDocument:keyup$={handler}
				onWindow:keyup$={handler}

				custom$={()=>console.log('custom')}
			/>
		)
	});
}, {
	tagName: "my-foo",
});
