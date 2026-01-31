import { component$ } from '@qwik.dev/core';

export const AtomStatus = component$(({ctx,atom})=>{
	let status = atom.status;
	if(!atom.real) {
		status="WILL-VANISH"
	} else if (JSON.stringify(atom.atom)==JSON.stringify(atom.real)) {
		status="WTFED"
	}
	return (
		<span title={atom.ID} onClick$={(ev)=>atomStatusClick(ctx,ev,[atom])} class={["atom",status,ctx.store[atom.ID]?"selected":null]}>
		</span>
	);
})
