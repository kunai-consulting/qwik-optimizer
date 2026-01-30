import { component$ } from "@qwik.dev/core";
const Cmp = component$(() => {
	const htmlSignal = useSignal("<h2><span>I'm a signal value!</span></h2>");
	return (
		<div>
			<div>
				<span id="first" dangerouslySetInnerHTML="vanilla HTML here" />
			</div>
			<div>
				<span id="second" dangerouslySetInnerHTML="<h1>I'm an h1!</h1>" class="after" />
			</div>
			<div>
				<span id="third" dangerouslySetInnerHTML={htmlSignal.value} class="after" />
				<button
					onClick$={() =>
						(htmlSignal.value = "<h2><span>I'm a updated signal value!</span></h2>")
					}
				></button>
			</div>
		</div>
	);
});
