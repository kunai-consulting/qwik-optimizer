import { component$, useComputed$, useTask$ } from '@qwik.dev/core'

export default component$(({ color, ...props }) => {
useComputed$(() => color)

useTask$(() => {
	props.checked
})

return 'hi'
})
