import { component$ } from "@qwik.dev/core";

// the count results in _fnSignal because of the rename
// would be nice to consider it a prop too
export default component$(({ message, id, count: c, ...rest }: Record<string, any>) => {
	const renders = useStore({ renders: 0 }, { reactive: false });
	renders.renders++;
	const rerenders = renders.renders + 0;
	return (
		<div id={id}>
			<span {...rest}>
			{message} {c}
			</span>
			<div class="renders">{rerenders}</div>
		</div>
	);
}
);
