import { component$, useTask$ } from '@qwik.dev/core'

export default component$<any>(({ ...props }) => {
useTask$(() => {
	props.checked
})

return 'hi'
})
